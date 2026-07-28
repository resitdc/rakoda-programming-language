use turbo_html2pdf_core::{
    CompileOptions, Diagnostics, EmitOptions, FontRegistry, NoImages, RenderInputs, build_cascade,
    compile, emit_pdf, render_pages, style::TokenSet,
};

fn main() {
    let html_content = r##"<!DOCTYPE html>
<html lang="id">
<head>
    <style>
        @page { size: A4 landscape; margin: 0; }
        .page-body { font-family: 'Georgia', serif; }
        .badge-wrapper { position: absolute; bottom: 60px; width: 100%; display: flex; justify-content: center; }
    </style>
</head>
<body>
    <div class="page-body">
        <div class="badge-wrapper">
            <svg width="110" height="110" viewBox="0 0 110 110">
                <circle cx="55" cy="55" r="45" fill="#fbbf24" stroke="#f59e0b" stroke-width="5" />
                <text x="55" y="48" font-family="Georgia, serif" font-size="14" font-weight="bold" fill="white" text-anchor="middle">LULUS</text>
                <text x="55" y="68" font-family="Georgia, serif" font-size="14" font-weight="bold" fill="white" text-anchor="middle">TERBAIK</text>
            </svg>
        </div>
    </div>
</body>
</html>"##;

    let mut author_css = String::new();
    if let (Some(start), Some(end)) = (html_content.find("<style>"), html_content.find("</style>"))
        && start < end
    {
        author_css = html_content[start + 7..end].to_string();
    }

    let (program, _diags) = compile(html_content, &CompileOptions::default()).unwrap();
    let data = serde_json::Value::Null;
    let (_nodes, _) = program.render_nodes(&data, Some(0)).unwrap();

    let cascade = build_cascade(&author_css, "", TokenSet::default());
    let fonts = FontRegistry::new();
    let rules = turbo_html2pdf_core::style::parse_stylesheet(&author_css).at_rules;

    let inputs = RenderInputs {
        program: &program,
        data: &data,
        cascade: &cascade,
        at_rules: &rules,
        fonts: &fonts,
        images: &NoImages,
        now: Some(0),
    };

    let mut diags = Diagnostics::default();
    let pages = render_pages(&inputs, &mut diags).unwrap();
    let pdf_bytes = emit_pdf(&pages, &EmitOptions::default());

    std::fs::write("test_styled.pdf", pdf_bytes).unwrap();
    println!("Generated test_styled.pdf");
}
