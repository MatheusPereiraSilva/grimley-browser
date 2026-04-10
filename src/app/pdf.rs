use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};

pub(crate) fn load_pdf_state(pdf_url: &str) -> (Option<String>, Option<String>) {
    match fetch_pdf_data_base64(pdf_url) {
        Ok(data) => (Some(data), None),
        Err(error) => (None, Some(error)),
    }
}

fn fetch_pdf_data_base64(pdf_url: &str) -> Result<String, String> {
    let response = reqwest::blocking::get(pdf_url)
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

    Ok(BASE64_STANDARD.encode(bytes))
}
