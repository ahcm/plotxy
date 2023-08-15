use colorgrad::Gradient;

use plotters::chart::ChartBuilder;
use plotters::coord::combinators::IntoLogRange;
use plotters::prelude::*;
use polars::prelude::*;
use std::error::Error;
use std::fmt::Debug;
use std::io::Cursor;
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
    /// file to save PNG plot to, default append .plotyy.png to input filename
    outfile: Option<PathBuf>,

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
            ".plotxy.png"
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

    let df = CsvReader::new(Cursor::new(buf))
        .with_delimiter(delimiter)
        .with_skip_rows(opt.skip)
        .has_header(opt.Header)
        .finish()
        .unwrap();

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
    let xdesc = &opt.xdesc;
    let ydesc = &opt.ydesc;
    let xdesc_area = opt.xdesc_area;
    let ydesc_area = opt.ydesc_area;
    let idx: Series = (0..df.height() as i64).collect();
    let x = if opt.x == 0 { &idx } else { &df[opt.x - 1] };
    let y = &df[opt.y - 1];
    let x_max: f64 = x
        .max()
        .expect("x is non numerical? If file has a header use -H or --skip");
    let y_max: f64 = y
        .max()
        .expect("y is non numerical? If file has a header use -H or --skip");
    let _y_min: f64 = y
        .min()
        .expect("y is non numerical? If file has a header use -H or --skip");
    let x_dim_min = opt.x_dim_min;
    let y_dim_min = opt.y_dim_min;
    let x_dim_max = opt.x_dim_max.unwrap_or(next_potence(x_max as f64));
    let y_dim_max = opt.y_dim_max.unwrap_or(next_potence(y_max as f64));

    let plot_color = hex::decode(&opt.plot_color).expect("Decoding failed");
    let plot_plotters_color = RGBColor(plot_color[0], plot_color[1], plot_color[2]);

    let root = BitMapBackend::new(&plot_filename, (opt.width, number_of_panels * opt.height))
        .into_drawing_area();
    root.fill(&WHITE)?;
    root.titled(opt.title.as_ref().unwrap_or(&plot_filename), ("sans-serif", 20))?;

    let xf64 = x.cast(&DataType::Float64).expect("cast");
    let yf64 = y.cast(&DataType::Float64).expect("cast");
    let xy = xf64
        .f64()
        .expect("x")
        .into_iter()
        .zip(yf64.f64().expect("y").into_iter());
    //.zip(df[3].i64().expect("facet as i64").into_iter())
    //    .zip(std::iter::repeat(1).map(|c| Some(c)))

    let blue_iter: Series = std::iter::repeat(1f64).take(df.height()).collect();

    let color_iterator: Vec<ShapeStyle> = if let Some(color_facet_index) = opt.color
    {
        df[color_facet_index - 1]
            .f64()
            .expect("facet as f64")
            .into_iter()
            .map(|c| ShapeStyle::from(Palette99::pick(c.unwrap_or(0.0f64) as usize)).filled())
            .collect()
    }
    else if let Some(color_gradient_index) = opt.gradient
    {
        get_gradient_color_iter(&df[color_gradient_index - 1])
    }
    else
    {
        blue_iter
            .f64()
            .expect("oops on blue iterator")
            .into_iter()
            .map(|_c| ShapeStyle::from(plot_plotters_color.mix(opt.alpha)).filled())
            .collect()
    };

    let shapes = xy
        .zip(color_iterator) //.zip(color_iterator)
        .map(|((x, y), c)| match (x, y)
        {
            (Some(xx), Some(yy)) => Circle::new((xx, yy), 5, c),
            _ =>
            {
                println!("NA value as 0 0");
                Circle::new((0.0, 0.0), 5, c)
            }
        });

    let panels = root.split_evenly((number_of_panels as usize, 1));
    let panel = &panels[0];
    let mut chart = ChartBuilder::on(panel);
    chart
        .x_label_area_size(xdesc_area)
        .y_label_area_size(ydesc_area)
        .margin(26u32);
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
                .label_style(("sans-serif", 24u32))
                .axis_desc_style(("sans-serif", 22u32))
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
                .label_style(("sans-serif", 24u32))
                .axis_desc_style(("sans-serif", 22u32))
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
                .label_style(("sans-serif", 24u32))
                .axis_desc_style(("sans-serif", 22u32))
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
                .label_style(("sans-serif", 24u32))
                .axis_desc_style(("sans-serif", 22u32))
                .draw()
                .expect("draw");

            grid.draw_series(shapes).expect("Backend Error");
        }
    }
    Ok(())
}

fn get_gradient_color_iter(column: &Series) -> Vec<ShapeStyle>
{
    let grad = colorgrad::GradientBuilder::new()
        .html_colors(&["yellow", "red"])
        .domain(&[column.min().unwrap_or(0.0), column.max().unwrap_or(1.0)])
        .build::<colorgrad::LinearGradient>()
        .expect("prebuilt gradient should always work");

    column
        .f64()
        .expect("facet as f64")
        .into_iter()
        .map(|c| {
            ShapeStyle::from(rbgcolor_from_gradient(grad.at(c.unwrap() as f32).to_rgba8())).filled()
        })
        .collect()
}

fn rbgcolor_from_gradient(g: [u8; 4]) -> RGBColor
{
    RGBColor(g[0], g[1], g[2])
}
