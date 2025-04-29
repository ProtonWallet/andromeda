///! [`PDFGenerator`] is responsible for generating PDF documents from Typst text.
///! It provides functionality to convert Typst text into PDF format using the Typst PDF library.
use crate::error::Error;
use crate::typst_wrapper::TypstWrapper;
use typst_pdf::PdfOptions;

#[derive(Clone)]
pub struct PDFGenerator {}

impl PDFGenerator {
    /// Creates a new instance of `PDFGenerator`.
    pub fn new() -> Self {
        Self {}
    }
}

impl PDFGenerator {
    /// generate pdf from typst text
    pub fn generate_pdf_from_typst_text(&self, typst_text: String) -> Result<Vec<u8>, Error> {
        // Create world with content.
        let world = TypstWrapper::new(typst_text);
        // Render document
        let document = typst::compile(&world).output.map_err(|_| Error::CompileTypst)?;
        // Output to pdf
        let pdf = typst_pdf::pdf(&document, &PdfOptions::default()).map_err(|_| Error::ExportPDF)?;

        Ok(pdf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_generate_pdf_from_typst_text() {
        let generator = PDFGenerator::new();
        let typst_text = "Hello world".to_string();
        let pdf_data = generator.generate_pdf_from_typst_text(typst_text).unwrap();
        assert!(!pdf_data.is_empty());
    }
}
