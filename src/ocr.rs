pub mod ocr {
    use ocr_rs::{OcrEngine, OcrResult_};
    use pdf2image::{PDF, RenderOptionsBuilder};

    pub fn ocr_to_string(text: Box<Vec<OcrResult_>>) -> anyhow::Result<String> {
        let mut txt = String::new();
        for line in *text {
            txt.push_str(
                format!(
                    "text: {}\n\
                position: left: {} top: {} width: {} height: {}\n\
                confidence: {}\n\n",
                    line.text,
                    line.bbox.rect.left(),
                    line.bbox.rect.top(),
                    line.bbox.rect.width(),
                    line.bbox.rect.height(),
                    line.confidence
                )
                .as_str(),
            );
        }
        return Ok(txt);
    }

    pub fn ocr_do(engine: &OcrEngine, path: &String) -> anyhow::Result<Box<Vec<OcrResult_>>> {
        let pdf = PDF::from_file(&path)?;
        let pages = pdf.render(
            pdf2image::Pages::Range(1..=8),
            RenderOptionsBuilder::default().build()?,
        )?;
        let text = engine.recognize(&pages[0])?;
        return Ok(Box::new(text));
    }
}
