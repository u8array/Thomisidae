use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use scraper::Html;
use std::fs;
use std::path::{Path, PathBuf};
use thomisidae::tools::fetch_text::{extract_best_blocks, extract_fallback_blocks};
#[cfg(feature = "readability")]
use thomisidae::tools::fetch_text::extract_readability as readability_extract;
#[cfg(feature = "readability")]
use url::Url;

fn load_fixture_path(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| sample_html())
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

fn list_html_fixtures_recursive() -> Vec<PathBuf> {
    let root = Path::new("benches/data");
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    let mut files: Vec<PathBuf> = Vec::new();

    while let Some(dir) = stack.pop() {
        if let Ok(read_dir) = fs::read_dir(&dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext.eq_ignore_ascii_case("html") || ext.eq_ignore_ascii_case("htm") {
                        files.push(path);
                    }
                }
            }
        }
    }

    files.sort();
    files
}

fn bench_fair(c: &mut Criterion) {
    // Discover all .html/.htm fixtures under benches/data (recursively); fallback to an in-memory sample when none.
    let fixtures: Vec<PathBuf> = list_html_fixtures_recursive();
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
        let root = Path::new("benches/data");
        for path in &fixtures {
            let html = load_fixture_path(path);
            let name_buf = path.strip_prefix(root).unwrap_or(path);
            let name = name_buf.to_string_lossy();

            // 1) best_blocks: extraction-only (DOM pre-parsed)
            let doc = Html::parse_document(&html);
            group.bench_with_input(BenchmarkId::new("best_blocks_extraction_only", &name), &name, |b, _| {
                b.iter(|| {
                    let blocks = extract_best_blocks(&doc).unwrap_or_else(|| extract_fallback_blocks(&doc));
                    black_box(blocks.join("\n"))
                })
            });

            // 2) best_blocks: parse + extract (end-to-end fair vs readability)
            group.bench_with_input(BenchmarkId::new("best_blocks_parse_and_extract", &name), &name, |b, _| {
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
                group.bench_with_input(BenchmarkId::new("readability_end_to_end", &name), &name, |b, _| {
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
