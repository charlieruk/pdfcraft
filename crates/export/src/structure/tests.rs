use super::*;
use crate::Block;

/// A block of 10-point text from `y0` to `y1`.
fn block(text: &str, y0: f64, y1: f64) -> Block {
    Block {
        text: text.into(),
        rect: [72.0, y0, 300.0, y1],
        size: 10.0,
        bold: false,
        italic: false,
        color: [0.0; 3],
        rotation: 0.0,
        space_around: [false; 2],
    }
}

/// A US Letter page: its margin bands are the 79.2 points at its top and bottom.
fn page(blocks: Vec<Block>) -> Page {
    Page { width: 612.0, height: 792.0, blocks, ..Page::default() }
}

fn running(pages: &[Page]) -> Vec<Vec<bool>> {
    structure(pages).into_iter().map(|s| s.running).collect()
}

#[test]
fn running_headers_and_page_numbers_repeat_in_the_margins() {
    let mut pages = vec![page(vec![block("XY123 Datasheet", 600.0, 624.0)])];
    for i in 1..5 {
        let mut blocks = vec![
            block("XY123 Low-noise amplifier", 740.0, 750.0),
            block(&format!("Page {} of 5", i + 1), 30.0, 40.0),
            block("Electrical characteristics", 700.0, 710.0),
        ];
        if i == 2 {
            blocks.push(block("Preliminary", 760.0, 770.0));
        }
        pages.push(page(blocks));
    }
    let found = running(&pages);
    assert_eq!(found[0], [false], "the title page has no header");
    for p in &found[1..] {
        assert_eq!(p[..3], [true, true, false], "the header and the page number, not the text below the margin");
    }
    assert!(!found[2][3], "a note on one page only");
}

#[test]
fn numbers_in_a_margin_must_follow_the_pages() {
    let lots = [page(vec![block("Lot 104", 30.0, 40.0)]), page(vec![block("Lot 230", 30.0, 40.0)])];
    assert_eq!(running(&lots), [[false], [false]]);
    let front = ["i", "ii", "iii", "iv"].map(|n| page(vec![block(n, 30.0, 40.0)]));
    assert_eq!(running(&front), [[true]; 4], "roman page numbers");
    let constant = [page(vec![block("Rev. 1.2  2026", 30.0, 40.0)]), page(vec![block("Rev. 1.2 2026", 30.0, 40.0)])];
    assert_eq!(running(&constant), [[true], [true]], "numbers that stay, spaces collapsed");
}

#[test]
fn a_continued_caption_in_the_margin_is_not_a_running_header() {
    let pages = [
        page(vec![block("Table 3. Pin functions", 600.0, 610.0)]),
        page(vec![block("Table 3. Pin functions (continued)", 740.0, 750.0), block("FIG. 2 (continued)", 30.0, 40.0)]),
        page(vec![block("Table 3. Pin functions (continued)", 740.0, 750.0), block("FIG. 2 (continued)", 30.0, 40.0)]),
    ];
    assert_eq!(running(&pages), [vec![false], vec![false, false], vec![false, false]]);
}

#[test]
fn text_repeated_on_few_pages_or_turned_is_not_running() {
    let mut pages: Vec<Page> = (0..10).map(|_| page(vec![block("Draft", 20.0, 30.0)])).collect();
    for p in &mut pages {
        p.blocks[0].rotation = 90.0;
    }
    for i in [3, 7] {
        pages[i].blocks.push(block("Notes", 740.0, 750.0));
    }
    let found = running(&pages);
    assert!(found.iter().all(|p| p.iter().all(|r| !r)), "{found:?}");
}

/// A run of `text` in `size` from `x0` to `x1`, its baseline at `base`.
fn run(text: &str, x0: f64, x1: f64, base: f64, size: f64) -> TextRun {
    TextRun {
        text: text.into(),
        rect: [x0, base - 0.2 * size, x1, base + 0.8 * size],
        words: Vec::new(),
        baseline: [x0, base],
        size,
        bold: false,
        italic: false,
        color: [0.0; 3],
        rotation: 0.0,
        block: Some(0),
    }
}

#[test]
fn smaller_runs_beside_a_line_above_or_below_its_baseline_are_scripts() {
    let mut turned = run("2", 100.0, 104.0, 304.0, 6.0);
    turned.rotation = 90.0;
    let runs = vec![
        run("E = mc", 72.0, 110.0, 700.0, 10.0),
        run("2", 110.0, 114.0, 704.0, 6.0),
        run("H", 72.0, 80.0, 650.0, 10.0),
        run("2", 80.0, 84.0, 648.0, 6.0),
        run("O", 84.0, 92.0, 650.0, 10.0),
        run("x", 72.0, 78.0, 600.0, 10.0),
        run("i", 78.0, 80.0, 598.0, 6.0),
        run("2", 78.0, 82.0, 604.0, 6.0),
        // A drop cap beside the first line it starts.
        run("T", 72.0, 90.0, 500.0, 24.0),
        run("he story", 92.0, 140.0, 512.0, 10.0),
        // Small capitals on the line's baseline.
        run("Note", 72.0, 95.0, 450.0, 10.0),
        run("SMALL", 96.0, 120.0, 450.0, 8.0),
        // Raised, but far along the line.
        run("foot", 72.0, 95.0, 400.0, 10.0),
        run("1", 300.0, 304.0, 404.0, 6.0),
        run("Turned", 72.0, 100.0, 300.0, 10.0),
        turned,
        run("long", 72.0, 95.0, 250.0, 10.0),
        run("a longer raised run", 95.0, 160.0, 254.0, 6.0),
    ];
    let pages = [Page { runs, ..page(Vec::new()) }];
    use Script::{Baseline as B, Sub, Super};
    assert_eq!(structure(&pages)[0].scripts, [B, Super, B, Sub, B, B, Sub, Super, B, B, B, B, B, B, B, B, B, B],);
}

#[test]
fn an_export_without_runs_has_no_scripts() {
    let pages = [page(vec![block("Text", 400.0, 410.0)])];
    assert_eq!(structure(&pages), [PageStructure { running: vec![false], scripts: Vec::new() }]);
}

#[test]
fn a_header_on_every_page_of_a_long_document_is_found_quickly() {
    let pages: Vec<Page> = (0..20_000).map(|i| page(vec![block("Header", 740.0, 750.0), block(&format!("{}", i + 1), 30.0, 40.0)])).collect();
    assert!(running(&pages).iter().all(|p| p == &[true, true]));
}
