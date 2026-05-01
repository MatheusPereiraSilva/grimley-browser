use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use super::model::PdfSource;

type PdfBinaryCache = Arc<Mutex<HashMap<String, Vec<u8>>>>;

pub(crate) type PdfFetcherHandle = Arc<PdfFetcher>;

pub(crate) struct PdfFetcher {
    cache: PdfBinaryCache,
    job_tx: Sender<PdfFetchJob>,
}

struct PdfFetchJob {
    source: PdfSource,
    response_tx: Sender<Result<Vec<u8>, String>>,
}

impl PdfFetcher {
    pub(crate) fn new() -> PdfFetcherHandle {
        let (job_tx, job_rx) = mpsc::channel();
        let cache = Arc::new(Mutex::new(HashMap::new()));

        spawn_pdf_fetch_worker(job_rx, Arc::clone(&cache));

        Arc::new(Self { cache, job_tx })
    }

    pub(crate) fn load_bytes(&self, source: &PdfSource) -> Result<Vec<u8>, String> {
        if let Some(bytes) = self.cached_bytes(source) {
            return Ok(bytes);
        }

        let (response_tx, response_rx) = mpsc::channel();
        self.job_tx
            .send(PdfFetchJob {
                source: source.clone(),
                response_tx,
            })
            .map_err(|_| {
                "O worker de PDF foi encerrado antes de concluir o download.".to_string()
            })?;

        response_rx
            .recv()
            .map_err(|_| "A resposta do worker de PDF foi interrompida.".to_string())?
    }

    fn cached_bytes(&self, source: &PdfSource) -> Option<Vec<u8>> {
        self.cache.lock().unwrap().get(&cache_key(source)).cloned()
    }
}

pub(crate) fn create_pdf_fetcher() -> PdfFetcherHandle {
    PdfFetcher::new()
}

pub(crate) fn fetch_pdf_bytes(
    pdf_fetcher: &PdfFetcherHandle,
    source: &PdfSource,
) -> Result<Vec<u8>, String> {
    pdf_fetcher.load_bytes(source)
}

fn spawn_pdf_fetch_worker(job_rx: Receiver<PdfFetchJob>, cache: PdfBinaryCache) {
    thread::Builder::new()
        .name("grimley-pdf-fetch".to_string())
        .spawn(move || run_pdf_fetch_worker(job_rx, cache))
        .expect("Nao foi possivel iniciar o worker de fetch de PDF");
}

fn run_pdf_fetch_worker(job_rx: Receiver<PdfFetchJob>, cache: PdfBinaryCache) {
    let client = reqwest::blocking::Client::builder()
        .build()
        .expect("Nao foi possivel criar o cliente HTTP de PDF");

    while let Ok(job) = job_rx.recv() {
        let cache_key = cache_key(&job.source);

        if let Some(bytes) = cache.lock().unwrap().get(&cache_key).cloned() {
            let _ = job.response_tx.send(Ok(bytes));
            continue;
        }

        let result = download_pdf_bytes(&client, &job.source);

        if let Ok(bytes) = &result {
            cache.lock().unwrap().insert(cache_key, bytes.clone());
        }

        let _ = job.response_tx.send(result);
    }
}

fn download_pdf_bytes(
    client: &reqwest::blocking::Client,
    source: &PdfSource,
) -> Result<Vec<u8>, String> {
    match source {
        PdfSource::RemoteUrl(pdf_url) => download_remote_pdf_bytes(client, pdf_url),
    }
}

fn download_remote_pdf_bytes(
    client: &reqwest::blocking::Client,
    pdf_url: &str,
) -> Result<Vec<u8>, String> {
    let response = client
        .get(pdf_url)
        .send()
        .map_err(|error| format!("Nao foi possivel baixar o PDF: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "O servidor respondeu com status {} ao baixar o PDF.",
            response.status()
        ));
    }

    let bytes = response
        .bytes()
        .map_err(|error| format!("Nao foi possivel ler os bytes do PDF: {error}"))?;

    Ok(bytes.to_vec())
}

fn cache_key(source: &PdfSource) -> String {
    match source {
        PdfSource::RemoteUrl(url) => url.clone(),
    }
}
