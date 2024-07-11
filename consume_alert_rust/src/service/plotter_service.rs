use crate::common::*;


const LINE_GRAPH: &str = "./data/line_graph.png";


pub fn draw_and_save_graph() -> Result<(), anyhow::Error> {
    
    // generate (x,y) points
    let x  = linspace(0.0, 5.0, 6);
    let y1: Vec<_> = x.iter().map(|v| 0.5 * *v).collect();
    let y2: Vec<_> = x.iter().map(|v| 1.0 * *v).collect();
    let y3: Vec<_> = x.iter().map(|v| 1.5 * *v).collect();
    let y4: Vec<_> = x.iter().map(|v| 2.0 * *v).collect();

    // configure and draw curves
    let mut curve1 = Curve::new();
    let mut curve2 = Curve::new();
    let mut curve3 = Curve::new();
    let mut curve4 = Curve::new();
    curve1.set_label("하 = 0.5 x");
    curve2.set_label("y = 1.0 x");
    curve3.set_label("y = 1.5 x");
    curve4.set_label("y = 2.0 x");
    curve1.draw(&x, &y1);
    curve2.draw(&x, &y2);
    curve3.draw(&x, &y3);
    curve4.draw(&x, &y4);

    // configure and draw legends
    let mut legend1 = Legend::new();
    legend1.set_fontsize(14.0)
        .set_handle_len(6.0)
        .set_num_col(2)
        .set_outside(true)
        .set_show_frame(false);
    legend1.draw();
    let mut legend2 = Legend::new();
    legend2.draw();

    // add curves and legends to subplots
    let mut plot = Plot::new();
    plot.set_subplot(2, 1, 1)
        .add(&curve1)
        .add(&curve2)
        .add(&curve3)
        .add(&curve4)
        .add(&legend1);
    plot.set_subplot(2, 1, 2)
        .add(&curve1)
        .add(&curve2)
        .add(&curve3)
        .add(&curve4)
        .add(&legend2);

    // save figure
    plot.save(LINE_GRAPH).unwrap();
    Ok(())

}


// pub fn test() {
//     // Example Text
//     let english = "Raster Text With Font";
//     let korean = "글꼴로 래스터 텍스트 사용";
//     let japanese = "フォント付きラスタテキスト";

//     // Init font, and paths.
//     let font = font::monospace_font();

//     // Render English Left Aligned.
//     let mut p = Plotter::new(Raster::with_clear(512, 512));
//     let mut r = Raster::with_clear(512, 512);
//     p.set_transform(Transform::with_scale(FONT_SIZE, FONT_SIZE));
//     let path = font.render(
//         english,
//         (512.0 - 64.0) / FONT_SIZE,
//         font::TextAlign::Left
//     ).0;
//     r.composite_matte(
//         (64, 0, 512, 512),
//         p.fill(FillRule::NonZero, path, Matte8::new(255)),
//         (),
//         Rgba8p::new(0, 0, 0, 255),
//         SrcOver,
//     );

//     // Render Korean Vertically
//     let mut pr = p.raster();
//     pr.clear();
//     p = Plotter::new(pr);
//     p.set_transform(Transform::with_scale(FONT_SIZE, FONT_SIZE));
//     let path = font.render(
//         korean,
//         512.0 / FONT_SIZE,
//         font::TextAlign::Vertical
//     ).0;
//     r.composite_matte(
//         (0, 0, 512, 512),
//         p.fill(FillRule::NonZero, path, Matte8::new(255)),
//         (),
//         Rgba8p::new(0, 0, 0, 255),
//         SrcOver,
//     );

//     // Render Japanese Vertically
//     let mut pr = p.raster();
//     pr.clear();
//     p = Plotter::new(pr);
//     p.set_transform(Transform::with_scale(FONT_SIZE, FONT_SIZE));
//     let path = font.render(
//         japanese,
//         (512.0 - 32.0 * 7.0) / FONT_SIZE,
//         font::TextAlign::Vertical
//     ).0;
//     r.composite_matte(
//         (32, 0, 512, 512),
//         p.fill(FillRule::NonZero, path, Matte8::new(255)),
//         (),
//         Rgba8p::new(0, 0, 0, 255),
//         SrcOver,
//     );

//     // Save PNG
//     let raster = Raster::<SRgba8>::with_raster(&r);
//     let mut out_data = Vec::new();
//     let mut encoder = Encoder::new(&mut out_data).into_step_enc();
//     encoder.still(&raster).expect("Failed to add frame");
//     std::fs::write("out.png", out_data).expect("Failed to save image");


// }

pub fn draw_test() -> Result<(), anyhow::Error> {

    // 그래프를 저장할 이미지 파일 경로 설정
    let root_area = BitMapBackend::new(LINE_GRAPH, (1000, 700)).into_drawing_area();
    root_area.fill(&WHITE)?;

    let start_time: DateTime<Utc> = match Utc.with_ymd_and_hms(2024, 1, 1, 21, 15, 33) {
        LocalResult::Single(start_time) => start_time,
        _ => return Err(anyhow!("test"))
    };
    
    let end_time = match Utc.with_ymd_and_hms(2024, 1, 1, 21, 20, 33) {
        LocalResult::Single(end_time) => end_time,
        _ => return Err(anyhow!("test"))
    };

    //let font: FontDesc = ("/Users/we/Documents/work_code/consume_alert_rust/consume_alert_rust/data/fonts/test.ttf", 20).into();
    
    let font = FontDesc::try_from(("Arial Unicode", 50, FontStyle::Normal)).unwrap_or_else(|_| {
        panic!("Font Error: Make sure font path and name are correct.");
    });

    let mut chart = ChartBuilder::on(&root_area)
        .caption("하하", font)
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(50)
        //.build_cartesian_2d(0..10, 0..50)?;
        //.build_cartesian_2d(0..10, 0..100)?;  
        .build_cartesian_2d(
            start_time..end_time,
            0..100)?;
    
    chart.configure_mesh()
        //.disable_y_mesh()
        .disable_x_mesh()
        //.disable_x_axis()
        .x_label_formatter(&|dt| dt.format("%H:%M:%S").to_string())
        .x_desc("Time (s)")
        .y_desc("Value (units)")
        .draw()?;
        
    let data1 = vec![
        (Utc.with_ymd_and_hms(2024, 1, 1, 21, 16, 33).unwrap(), 30),
        (Utc.with_ymd_and_hms(2024, 1, 1, 21, 17, 33).unwrap(), 10),
        (Utc.with_ymd_and_hms(2024, 1, 1, 21, 18, 33).unwrap(), 0),
        (Utc.with_ymd_and_hms(2024, 1, 1, 21, 19, 33).unwrap(), 40),
    ];


    let data2 = vec![
        (Utc.with_ymd_and_hms(2024, 1, 1, 21, 16, 33).unwrap(), 40),
        (Utc.with_ymd_and_hms(2024, 1, 1, 21, 17, 33).unwrap(), 50),
        (Utc.with_ymd_and_hms(2024, 1, 1, 21, 18, 33).unwrap(), 11),
        (Utc.with_ymd_and_hms(2024, 1, 1, 21, 19, 33).unwrap(), 77),
    ];

    // chart.draw_series(LineSeries::new(
    //     data1.iter().map(|(dt, value)| (*dt, *value)), &RED
    // ))?;

    // chart.draw_series(LineSeries::new(
    //     data2.iter().map(|(dt, value)| (*dt, *value)), &BLUE
    // ))?;
    
    // chart.draw_series(LineSeries::new(
    //     (0..10).map(|x| (x, x * x)), &RED,
    // ))?;
    
    // 여러 데이터 세트 정의
    // let data1 = vec![(0, 10), (1, 20), (2, 40), (3, 60), (4, 80)];
    // let data2 = vec![(0, 20), (1, 40), (2, 60), (3, 80), (4, 100)];
    // let data3 = vec![(0, 30), (1, 50), (2, 70), (3, 90), (4, 100)];

    // // 데이터 세트별로 선 그래프 그리기
    chart.draw_series(LineSeries::new(data1, &RED))?.label("Data Series 1").legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    chart.draw_series(LineSeries::new(data2, &BLUE))?.label("Data Series 2").legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
    // chart.draw_series(LineSeries::new(data3, &GREEN))?;

    chart.configure_series_labels()
        .position(SeriesLabelPosition::UpperLeft)
        .border_style(&BLACK).draw()?;
        //.background_style(&BLACK.mix(0.8))
        //.border_style(&BLACK);
    
    // 이미지 파일로 그래프 저장
    root_area.present()?;
    //println!("Line chart has been saved to 'plotters-doc-data/line_chart.png'.");
    Ok(())
}