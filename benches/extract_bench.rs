use criterion::{criterion_group, criterion_main, Criterion, black_box};

#[cfg(feature = "readability")]
use thomisidae::tools::fetch_text::extract_readability as readability_extract;
use thomisidae::tools::fetch_text::{extract_best_blocks, extract_fallback_blocks};
use scraper::Html;

fn sample_html() -> String {
    r#"
    <html><head><title>Example</title></head>
    <body>
      <header>Nav</header>
      <main>
        <article>
          <h1>Heading</h1>
          <p>This is a test paragraph with some content to assess extraction quality.</p>
          <p>Another paragraph follows to increase size and signal density.</p>
        </article>
        <aside>Ads</aside>
      </main>
    </body></html>
    "#.to_string()
}

fn bench_extract(c: &mut Criterion) {
    let html = sample_html();
    let doc = Html::parse_document(&html);

    c.bench_function("best_blocks", |b| {
        b.iter(|| {
            let blocks = extract_best_blocks(&doc).unwrap_or_else(|| extract_fallback_blocks(&doc));
            black_box(blocks.join("\n"))
        })
    });

    #[cfg(feature = "readability")]
    c.bench_function("readability", |b| {
        b.iter(|| {
            let out = readability_extract(&html).unwrap_or_default();
            black_box(out)
        })
    });
}

criterion_group!(benches, bench_extract);
criterion_main!(benches);
