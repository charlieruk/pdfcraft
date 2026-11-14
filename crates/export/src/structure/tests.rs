use super::*;
use crate::{Block, Drawing, Image};

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
    assert_eq!(structure(&pages), [PageStructure { running: vec![false], ..PageStructure::default() }]);
}

#[test]
fn a_header_on_every_page_of_a_long_document_is_found_quickly() {
    let pages: Vec<Page> = (0..20_000).map(|i| page(vec![block("Header", 740.0, 750.0), block(&format!("{}", i + 1), 30.0, 40.0)])).collect();
    assert!(running(&pages).iter().all(|p| p == &[true, true]));
}

#[test]
fn captions_start_with_a_label_and_a_number() {
    let label = |t: &str| figures::caption_label(t).map(|(k, n)| (k, n.to_owned()));
    use CaptionKind::{Figure as F, Table as T};
    for (text, want) in [
        ("Figure 3. Typical application", Some((F, "3"))),
        ("Table 2-1 Pin descriptions", Some((T, "2-1"))),
        ("FIG. 12: Timing", Some((F, "12"))),
        ("Fig 4b \u{2014} Detail", Some((F, "4b"))),
        ("Table A.4", Some((T, "A.4"))),
        ("  Table 3 (continued)", Some((T, "3"))),
        ("Figure 3 shows the circuit.", None),
        ("Figure 3, which", None),
        ("Tables 3 and 4", None),
        ("Figure3", None),
        ("Figurehead 2", None),
        ("Table of contents", None),
    ] {
        assert_eq!(label(text), want.map(|(k, n)| (k, n.to_owned())), "{text}");
    }
}

#[test]
fn a_caption_that_repeats_an_earlier_one_continues_it() {
    let pages = [
        page(vec![block("Table 3. Pin functions", 600.0, 610.0)]),
        page(vec![
            block("Table 3. Pin functions (continued)", 700.0, 710.0),
            block("Figure 3 shows the circuit.", 500.0, 510.0),
            block("Figure 4. Block diagram", 300.0, 310.0),
        ]),
    ];
    let found: Vec<Vec<Caption>> = structure(&pages).into_iter().map(|s| s.captions).collect();
    let caption = |block, kind, number: &str, continued| Caption { block, kind, number: number.into(), continued };
    assert_eq!(
        found,
        [
            vec![caption(0, CaptionKind::Table, "3", false)],
            vec![caption(0, CaptionKind::Table, "3", true), caption(2, CaptionKind::Figure, "4", false)],
        ]
    );
}

fn drawing(rect: [f64; 4]) -> Drawing {
    Drawing { rect, stroked: true, filled: false }
}

fn image(rect: [f64; 4]) -> Image {
    Image { ext: "png", bytes: std::sync::Arc::from(Vec::new()), rect, size: [rect[2] - rect[0], rect[3] - rect[1]], rotation: 0.0 }
}

fn at(text: &str, rect: [f64; 4]) -> Block {
    Block { rect, ..block(text, 0.0, 0.0) }
}

#[test]
fn a_figure_is_the_drawing_and_labels_next_to_its_caption() {
    let blocks = vec![
        at("A paragraph of body text above the chart, long enough to be text rather than a label of it.", [72.0, 600.0, 540.0, 640.0]),
        at("100", [80.0, 550.0, 98.0, 560.0]),
        at("25", [118.0, 388.0, 130.0, 398.0]),
        at("Temperature (\u{b0}C)", [200.0, 372.0, 300.0, 382.0]),
        at("Figure 2. Output power", [100.0, 350.0, 300.0, 362.0]),
        at("Text under the caption", [100.0, 300.0, 300.0, 310.0]),
    ];
    let mut background = drawing([0.0, 0.0, 612.0, 792.0]);
    background.filled = true;
    let drawings = vec![
        background,
        drawing([100.0, 400.0, 400.0, 560.0]),
        drawing([120.0, 400.0, 140.0, 500.0]),
        drawing([160.0, 400.0, 180.0, 450.0]),
        drawing([72.0, 100.0, 540.0, 100.0]),
    ];
    let pages = [Page { drawings, images: vec![image([400.0, 150.0, 500.0, 250.0])], ..page(blocks) }];
    let s = &structure(&pages)[0];
    assert_eq!(
        s.figures,
        [Figure { caption: 0, rect: [80.0, 372.0, 400.0, 560.0], drawings: vec![1, 2, 3], images: Vec::new(), labels: vec![1, 2, 3] }],
        "the axis title joins through the tick label; the background, the footer rule and the picture without a caption stay out"
    );
}

#[test]
fn each_caption_takes_the_nearest_free_drawing() {
    let blocks = vec![at("Figure 1. Above", [100.0, 580.0, 300.0, 590.0]), at("Figure 2. Below", [100.0, 450.0, 300.0, 460.0])];
    let drawings = vec![drawing([100.0, 600.0, 300.0, 700.0]), drawing([100.0, 470.0, 300.0, 570.0])];
    let s = &structure(&[Page { drawings, ..page(blocks) }])[0];
    assert_eq!(s.figures.iter().map(|f| (f.caption, f.drawings.clone())).collect::<Vec<_>>(), [(0, vec![0]), (1, vec![1])]);

    let blocks = vec![at("Figure 5: A picture under its caption", [100.0, 700.0, 300.0, 710.0])];
    let s = &structure(&[Page { images: vec![image([100.0, 600.0, 300.0, 690.0])], ..page(blocks) }])[0];
    assert_eq!(s.figures.iter().map(|f| (f.caption, f.images.clone())).collect::<Vec<_>>(), [(0, vec![0])]);

    let blocks = vec![at("Figure 6. Too far", [100.0, 450.0, 300.0, 460.0]), at("Figure 7. Beside, not under", [400.0, 580.0, 550.0, 590.0])];
    let s = &structure(&[Page { drawings: vec![drawing([100.0, 600.0, 300.0, 700.0])], ..page(blocks) }])[0];
    assert!(s.figures.is_empty(), "{:?}", s.figures);

    let blocks = vec![at("Figure 10. Nearer", [100.0, 590.0, 300.0, 598.0]), at("Figure 11. Further", [100.0, 710.0, 300.0, 720.0])];
    let s = &structure(&[Page { drawings: vec![drawing([100.0, 600.0, 300.0, 700.0])], ..page(blocks) }])[0];
    assert_eq!(s.figures.iter().map(|f| f.caption).collect::<Vec<_>>(), [0], "one drawing, one caption");
}

#[test]
fn a_ruled_table_is_not_a_figure() {
    let mut blocks = Vec::new();
    let mut rules = Vec::new();
    for (r, y) in [700.0, 680.0, 660.0].into_iter().enumerate() {
        for (c, x) in [100.0, 200.0, 300.0].into_iter().enumerate() {
            blocks.push(at(&format!("r{r}c{c}"), [x + 4.0, y + 4.0, x + 40.0, y + 14.0]));
        }
    }
    for y in [720.0, 700.0, 680.0, 660.0] {
        rules.push([100.0, y, 400.0, y]);
    }
    for x in [100.0, 200.0, 300.0, 400.0] {
        rules.push([x, 660.0, x, 720.0]);
    }
    assert_eq!(crate::tables(&blocks, &rules).0.len(), 1, "the grid is a table");
    blocks.push(at("Figure 8. Under a table", [100.0, 640.0, 300.0, 650.0]));
    let drawings = rules.iter().map(|&r| drawing(r)).collect();
    let s = &structure(&[Page { drawings, rules, ..page(blocks) }])[0];
    assert_eq!(s.captions.len(), 1);
    assert!(s.figures.is_empty(), "{:?}", s.figures);
}

#[test]
fn a_page_of_scattered_paths_gives_up_on_figures_within_its_budget() {
    // Dots further apart than the paths of one drawing are.
    let dot = |i: usize| [(i % 100) as f64 * 13.0, (i / 100) as f64 * 13.0];
    let drawings = (0..6000).map(dot).map(|[x, y]| drawing([x, y, x + 0.5, y + 0.5])).collect();
    let pages = [Page { drawings, ..page(vec![at("Figure 9. Dots", [100.0, 780.0, 300.0, 790.0])]) }];
    assert!(structure(&pages)[0].figures.is_empty());
}
