pub(crate) fn fetch_pdf_bytes(pdf_url: &str) -> Result<Vec<u8>, String> {
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

    Ok(bytes.to_vec())
}
