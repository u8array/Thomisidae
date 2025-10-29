use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use scraper::Html;
use std::fs;
use std::path::{Path, PathBuf};
use thomisidae::tools::fetch_text::{extract_best_blocks, extract_fallback_blocks};
#[cfg(feature = "readability")]
use thomisidae::tools::fetch_text::extract_readability as readability_extract;
#[cfg(feature = "readability")]
use url::Url;

fn load_fixture(name: &str) -> String {
    let p = Path::new("benches/data").join(name);
    fs::read_to_string(&p).unwrap_or_else(|_| sample_html())
}

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

fn list_html_fixtures() -> Vec<String> {
    let dir = Path::new("benches/data");
    let mut files: Vec<String> = Vec::new();
    if let Ok(read_dir) = fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path: PathBuf = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("html") || ext.eq_ignore_ascii_case("htm") {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        files.push(name.to_string());
                    }
                }
            }
        }
        files.sort();
    }
    files
}

fn bench_fair(c: &mut Criterion) {
    // Discover all .html/.htm fixtures under benches/data; fallback to an in-memory sample when none.
    let fixtures: Vec<String> = list_html_fixtures();
    let mut group = c.benchmark_group("extract_fair");
    // Optionally tune sampling
    // group.sample_size(50);

    if fixtures.is_empty() {
        // Run a single synthetic case to keep the suite functional without files
        let name = "synthetic_sample";
        let html = sample_html();

        // 1) best_blocks: extraction-only (DOM pre-parsed)
        let doc = Html::parse_document(&html);
        group.bench_with_input(BenchmarkId::new("best_blocks_extraction_only", name), name, |b, _| {
            b.iter(|| {
                let blocks = extract_best_blocks(&doc).unwrap_or_else(|| extract_fallback_blocks(&doc));
                black_box(blocks.join("\n"))
            })
        });

        // 2) best_blocks: parse + extract (end-to-end fair vs readability)
        group.bench_with_input(BenchmarkId::new("best_blocks_parse_and_extract", name), name, |b, _| {
            b.iter(|| {
                let doc2 = Html::parse_document(&html);
                let blocks = extract_best_blocks(&doc2).unwrap_or_else(|| extract_fallback_blocks(&doc2));
                black_box(blocks.join("\n"))
            })
        });

        // 3) readability: end-to-end (parse is internal)
        #[cfg(feature = "readability")]
        {
            let url = Url::parse("https://example.com/").unwrap();
            group.bench_with_input(BenchmarkId::new("readability_end_to_end", name), name, |b, _| {
                b.iter(|| {
                    let out = readability_extract(&html, &url).unwrap_or_default();
                    black_box(out)
                })
            });
        }
    } else {
        for name in &fixtures {
            let html = load_fixture(name);

            // 1) best_blocks: extraction-only (DOM pre-parsed)
            let doc = Html::parse_document(&html);
            group.bench_with_input(BenchmarkId::new("best_blocks_extraction_only", name), name, |b, _| {
                b.iter(|| {
                    let blocks = extract_best_blocks(&doc).unwrap_or_else(|| extract_fallback_blocks(&doc));
                    black_box(blocks.join("\n"))
                })
            });

            // 2) best_blocks: parse + extract (end-to-end fair vs readability)
            group.bench_with_input(BenchmarkId::new("best_blocks_parse_and_extract", name), name, |b, _| {
                b.iter(|| {
                    let doc2 = Html::parse_document(&html);
                    let blocks = extract_best_blocks(&doc2).unwrap_or_else(|| extract_fallback_blocks(&doc2));
                    black_box(blocks.join("\n"))
                })
            });

            // 3) readability: end-to-end (parse is internal)
            #[cfg(feature = "readability")]
            {
                let url = Url::parse("https://example.com/").unwrap();
                group.bench_with_input(BenchmarkId::new("readability_end_to_end", name), name, |b, _| {
                    b.iter(|| {
                        let out = readability_extract(&html, &url).unwrap_or_default();
                        black_box(out)
                    })
                });
            }
        }
    }

    group.finish();
}

criterion_group!(benches, bench_fair);
criterion_main!(benches);
