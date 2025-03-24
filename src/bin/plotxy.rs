use colorgrad::Gradient;

use plotters::chart::ChartBuilder;
use plotters::element::{Drawable, PointCollection};
use plotters::prelude::*;

use polars::prelude::*;
use std::error::Error;
use std::fmt::Debug;
use std::future::ready;
use std::io::Cursor;
use std::iter::Zip;
use std::path::PathBuf;
use structopt::StructOpt;

#[allow(non_snake_case)]
#[derive(Debug, StructOpt)]
#[structopt(name = "plotxy", about = "Plots tabular data", rename_all = "verbatim")]
struct Opt
{
    #[structopt(parse(from_os_str))]
    /// optional file with on entry per line [default: STDIN]
    input: Option<PathBuf>,

    #[structopt(long, short, default_value = "1")]
    /// column index to be used as X
    x: usize,

    #[structopt(long, short, default_value = "0.3")]
    /// transparancy channel
    alpha: f64,

    #[structopt(long, short, default_value = "1E88E5")]
    /// default plot color
    plot_color: String,

    #[structopt(long, short, default_value = "2")]
    /// column index to be used as Y
    y: usize,

    #[structopt(long, short)]
    /// column index to be used as color facet
    color: Option<usize>,

    #[structopt(long)]
    /// column index to be used as color gradient facet
    gradient: Option<usize>,

    // r"" makes it prinable as escaped in default
    #[structopt(short, long, default_value = r"\t")]
    /// column delimiter
    delimiter: String,

    #[structopt(long, short)]
    /// input has header line (see also --skip)
    Header: bool,

    #[structopt(long, short, default_value = "0")]
    /// skip lines before header
    skip: usize,

    #[structopt(long, short)]
    /// plot logarithmic Y-axis
    logx: bool,

    #[structopt(long, short)]
    /// plot logarithmic Y-axis
    logy: bool,

    #[structopt(long, default_value = "0.0")]
    /// minimum X dimension
    x_dim_min: f64,

    #[structopt(long)]
    /// maximum X dimension
    x_dim_max: Option<f64>,

    #[structopt(long, default_value = "0.0")]
    /// minimum Y dimension
    y_dim_min: f64,

    #[structopt(long)]
    /// maximum Y dimension
    y_dim_max: Option<f64>,

    #[structopt(parse(from_os_str), long, short)]
    /// file to save PNG plot to, default append .plotxy.png to input filename
    outfile: Option<PathBuf>,

    #[structopt(long)]
    /// set output format to svg
    svg: bool,

    #[structopt(short, long)]
    /// title above the plot, default filename
    title: Option<String>,

    #[structopt(long, default_value = "2560")]
    /// image width
    width: u32,

    #[structopt(long, default_value = "1200")]
    /// image width
    height: u32,

    #[structopt(long, default_value = "X")]
    /// x-axis label
    xdesc: String,

    #[structopt(long, default_value = "Y")]
    /// y-axis label
    ydesc: String,

    #[structopt(long, default_value = "70")]
    /// x-axis label area size
    xdesc_area: u32,

    #[structopt(long, default_value = "100")]
    /// y-axis label area size
    ydesc_area: u32,

    #[structopt(long, default_value = "sans-serif")]
    /// label font name
    label_font: String,

    #[structopt(long, default_value = "24")]
    /// label font size
    label_font_size: u32,

    #[structopt(long, default_value = "sans-serif")]
    /// axis description font name
    axis_desc_font: String,

    #[structopt(long, default_value = "22")]
    /// axis description font size
    axis_desc_font_size: u32,

    #[structopt(long, default_value = "sans-serif")]
    /// title font name
    title_font: String,

    #[structopt(long, default_value = "24")]
    /// title font size
    title_font_size: u32,

    #[structopt(long, default_value = "3")]
    /// point size, radius
    point_size: u32,

    #[structopt(long, default_value = "circle")]
    /// plotting shape: circle, column
    shape: String,
}

fn main() -> std::result::Result<(), Box<dyn Error>>
{
    let mut opt = Opt::from_args();

    let mut input: Box<dyn std::io::Read + 'static> = if let Some(path) = &opt.input
    {
        Box::new(std::fs::File::open(path).unwrap())
    }
    else
    {
        opt.input = Some(std::path::Path::new("STDIN").to_path_buf());
        Box::new(std::io::stdin())
    };

    if opt.outfile.is_none()
    {
        let mut outname = PathBuf::new();
        outname.set_file_name(format!(
            "{}{}",
            opt.input
                .as_ref()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy(),
            if opt.svg
            {
                ".plotxy.svg"
            }
            else
            {
                ".plotxy.png"
            }
        ));
        opt.outfile = Some(outname)
    }

    // accept escaped delimiters
    // could be expanded to aliases e.g. "TAB"
    let delimiter = match opt.delimiter.as_str()
    {
        r"\t" => b'\t', // structopt needs r"" to show default as escaped, also for sepcifiying as escaped in CLI
        _ => *opt
            .delimiter
            .as_bytes()
            .first()
            .expect("Not a valid delimiter"),
    };

    // XXX stdin is really hard to use for CsvReader, so slurp the file
    let mut buf = Vec::new();
    input.read_to_end(&mut buf).expect("Error reading input");

    
    let csv_parse_options = CsvParseOptions::default()
        .with_separator(delimiter)
        .with_try_parse_dates(false)
        .with_missing_is_null(true)
        .with_truncate_ragged_lines(true);

    let csv_read_options = CsvReadOptions::default()
        .with_parse_options(csv_parse_options)
        .with_ignore_errors(true)
        .with_skip_rows(opt.skip)
        .with_has_header(opt.Header);

    let df = csv_read_options.into_reader_with_file_handle(Cursor::new(buf)).finish().unwrap();

    plot_xy(&opt, df)
}

fn next_potence(x: f64) -> f64
{
    10f64.powf(((x.log10() * 10f64).ceil()) / 10.0)
}

fn plot_xy(opt: &Opt, df: DataFrame) -> std::result::Result<(), Box<dyn Error>>
{
    let plot_filename = opt
        .outfile
        .as_ref()
        .expect("Outfile missing")
        .to_str()
        .unwrap()
        .to_string();

    println!("{}", plot_filename);

    let number_of_panels = 1;

    if opt.svg
    {
        plot_on_backend(
            opt,
            df,
            SVGBackend::new(&plot_filename, (opt.width, number_of_panels * opt.height)),
        );
    }
    else
    {
        plot_on_backend(
            opt,
            df,
            BitMapBackend::new(&plot_filename, (opt.width, number_of_panels * opt.height)),
        );
    }
    Ok(())
}

fn plot_on_backend<'a, B>(opt: &Opt, df: DataFrame, backend: B)
where
    B: DrawingBackend,
{
    let plot_filename = opt
        .outfile
        .as_ref()
        .expect("Outfile missing")
        .to_str()
        .unwrap()
        .to_string();

    let root = Box::new(backend.into_drawing_area());
    root.fill(&WHITE).expect("root.fill failed");
    root.titled(
        opt.title.as_ref().unwrap_or(&plot_filename),
        (opt.title_font.as_str(), opt.title_font_size),
    )
    .expect("root.titled failed");

    let number_of_panels = 1;
    let panels = root.split_evenly((number_of_panels as usize, 1));
    let panel = &panels[0];
    let mut chart = ChartBuilder::on(&panel);
    let xdesc_area = opt.xdesc_area;
    let ydesc_area = opt.ydesc_area;
    chart
        .x_label_area_size(xdesc_area)
        .y_label_area_size(ydesc_area)
        .margin(26u32);

    let idx: Series = (0..df.height() as i64).collect();
    let x = if opt.x == 0 { &idx } else { &df[opt.x - 1].as_series().unwrap() };
    let y = &df[opt.y - 1].as_series().unwrap();
    let x_max: f64 = x
        .max()
        .expect("x is non numerical? If file has a header use -H or --skip")
        .expect("some data for x");
    let y_max: f64 = y
        .max()
        .expect("y is non numerical? If file has a header use -H or --skip")
        .expect("some data for y");
    let _y_min: f64 = y
        .min()
        .expect("y is non numerical? If file has a header use -H or --skip")
        .expect("some data for y");

    let xf64 = x.cast(&DataType::Float64).expect("cast");
    let yf64 = y.cast(&DataType::Float64).expect("cast");
    let xyc = make_xyc(&xf64, &yf64, &df, &opt);

    match opt.shape.as_str()
    {
        "column" =>
        {
            let shapes = xyc.map(|((x, y), c)| match (x, y)
            {
                (Some(xx), Some(yy)) => Rectangle::new([(xx - 0.4f64, yy), (xx + 0.4f64, 0f64)], c),
                _ =>
                {
                    println!("NA value as 0 0");
                    Rectangle::new([(0.0, 0.0), (0.0, 0.0)], c)
                }
            });
            plot_shapes(&mut chart, shapes, &opt, x_max, y_max);
        }
        _ =>
        {
            let shapes = xyc.map(|((x, y), c)| match (x, y)
            {
                (Some(xx), Some(yy)) => Circle::new((xx, yy), opt.point_size, c),
                _ =>
                {
                    println!("NA value as 0 0");
                    Circle::new((0.0, 0.0), opt.point_size, c)
                }
            });
            plot_shapes(&mut chart, shapes, &opt, x_max, y_max);
        }
    }
}

/// Returns an iterator over x/y points and the color based on facet/gradient
fn make_xyc<'a, 'b>(
    x: &'a Series,
    y: &'b Series,
    df: &DataFrame,
    opt: &Opt,
) -> Zip<
    Zip<
        Box<dyn PolarsIterator<Item = Option<f64>> + 'a>,
        Box<dyn PolarsIterator<Item = Option<f64>> + 'b>,
    >,
    std::vec::IntoIter<ShapeStyle>,
>
{
    let plot_color = hex::decode(&opt.plot_color).expect("Decoding failed");
    let plot_plotters_color = RGBColor(plot_color[0], plot_color[1], plot_color[2]);
    let xy = x
        .f64()
        .expect("x")
        .into_iter()
        .zip(y.f64().expect("y").into_iter());

    let color_iterator = if let Some(color_facet_index) = opt.color
    {
        df[color_facet_index - 1]
            .cast(&DataType::Categorical(None, CategoricalOrdering::Lexical))
            .expect("cast to Categorial failed")
            .cast(&DataType::Float64)
            .expect("cast to f64 failed")
            .f64()
            .expect("facet as f64")
            .into_iter()
            .map(|c| ShapeStyle::from(Palette99::pick(c.expect("color as f64") as usize)).filled())
            .collect()
    }
    else if let Some(color_gradient_index) = opt.gradient
    {
        get_gradient_color_iter(&opt, &df[color_gradient_index - 1].as_series().unwrap())
    }
    else
    {
        (0..xy.len())
            .into_iter()
            .map(|_c| ShapeStyle::from(plot_plotters_color.mix(opt.alpha)).filled())
            .collect()
    };
    xy.zip(color_iterator)
}

fn plot_shapes<'a, 'b, DB, T>(
    chart: &mut ChartBuilder<'a, 'b, DB>,
    shapes: T,
    opt: &Opt,
    x_max: f64,
    y_max: f64,
) where
    DB: DrawingBackend,
    T: IntoIterator,
    T::Item: Drawable<DB>,
    for<'d> &'d <T as IntoIterator>::Item: PointCollection<'d, (f64, f64)>,
{
    let xdesc = &opt.xdesc;
    let ydesc = &opt.ydesc;
    let x_dim_min = opt.x_dim_min;
    let y_dim_min = opt.y_dim_min;
    let x_dim_max = opt.x_dim_max.unwrap_or(next_potence(x_max as f64));
    let y_dim_max = opt.y_dim_max.unwrap_or(next_potence(y_max as f64));
    if opt.logx
    {
        if opt.logy
        {
            let mut grid = chart
                .build_cartesian_2d(
                    (x_dim_min..x_dim_max).log_scale(),
                    (y_dim_min..y_dim_max).log_scale(),
                )
                .expect("grid");
            grid.configure_mesh()
                .disable_x_mesh()
                .bold_line_style(WHITE.mix(0.3))
                .y_desc(ydesc)
                .x_desc(xdesc)
                .label_style((opt.label_font.as_str(), opt.label_font_size))
                .axis_desc_style((opt.axis_desc_font.as_str(), opt.axis_desc_font_size))
                .draw()
                .expect("draw");

            grid.draw_series(shapes).expect("Backend Error");
        }
        else
        {
            let mut grid = chart
                .build_cartesian_2d((x_dim_min..x_dim_max).log_scale(), y_dim_min..y_dim_max)
                .expect("grid");
            grid.configure_mesh()
                .disable_x_mesh()
                .bold_line_style(WHITE.mix(0.3))
                .y_desc(ydesc)
                .x_desc(xdesc)
                .label_style((opt.label_font.as_str(), opt.label_font_size))
                .axis_desc_style((opt.axis_desc_font.as_str(), opt.axis_desc_font_size))
                .draw()
                .expect("draw");

            grid.draw_series(shapes).expect("Backend Error");
        }
    }
    else
    {
        if opt.logy
        {
            let mut grid = chart
                .build_cartesian_2d(x_dim_min..x_dim_max, (y_dim_min..y_dim_max).log_scale())
                .expect("grid");
            grid.configure_mesh()
                .disable_x_mesh()
                .bold_line_style(WHITE.mix(0.3))
                .y_desc(ydesc)
                .x_desc(xdesc)
                .label_style((opt.label_font.as_str(), opt.label_font_size))
                .axis_desc_style((opt.axis_desc_font.as_str(), opt.axis_desc_font_size))
                .draw()
                .expect("draw");

            grid.draw_series(shapes).expect("Backend Error");
        }
        else
        {
            let mut grid = chart
                .build_cartesian_2d(x_dim_min..x_dim_max, y_dim_min..y_dim_max)
                .expect("grid");
            grid.configure_mesh()
                .disable_x_mesh()
                .bold_line_style(WHITE.mix(0.3))
                .y_desc(ydesc)
                .x_desc(xdesc)
                .label_style((opt.label_font.as_str(), opt.label_font_size))
                .axis_desc_style((opt.axis_desc_font.as_str(), opt.axis_desc_font_size))
                .draw()
                .expect("draw");

            grid.draw_series(shapes).expect("Backend Error");
        }
    }
}

fn get_gradient_color_iter(opt: &Opt, series: &Series) -> Vec<ShapeStyle>
{
    let values = series.f32().expect("failed to convert gradient color column into f32");
    let grad = colorgrad::GradientBuilder::new()
        .html_colors(&["yellow", "red"])
        .domain(&[values.min().unwrap(), values.max().unwrap()])
        .build::<colorgrad::LinearGradient>()
        .expect("prebuilt gradient should always work");

    values
        .into_iter()
        .map(|c| {
            ShapeStyle::from(
                rbgcolor_from_gradient(grad.at(c.unwrap() as f32).to_rgba8(), opt.alpha).filled(),
            )
        })
        .collect()
}

fn rbgcolor_from_gradient(g: [u8; 4], alpha: f64) -> RGBAColor
{
    RGBAColor(g[0], g[1], g[2], alpha)
}
