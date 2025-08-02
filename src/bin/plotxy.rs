use colorgrad::Gradient;

use plotters::chart::ChartBuilder;
use plotters::element::{Drawable, PointCollection};
use plotters::prelude::*;

use polars::prelude::*;
use std::error::Error;
use std::io::Cursor;
use std::iter::Zip;
use std::path::PathBuf;

#[derive(Debug)]
enum PlotError
{
    IoError(std::io::Error),
    PolarsError(PolarsError),
    HexDecodeError(hex::FromHexError),
    InvalidColumn(String),
    InvalidData(String),
}

impl std::fmt::Display for PlotError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            PlotError::IoError(e) => write!(f, "IO error: {}", e),
            PlotError::PolarsError(e) => write!(f, "Data processing error: {}", e),
            PlotError::HexDecodeError(e) => write!(f, "Invalid color format: {}", e),
            PlotError::InvalidColumn(msg) => write!(f, "Invalid column: {}", msg),
            PlotError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
        }
    }
}

impl Error for PlotError {}

impl From<std::io::Error> for PlotError
{
    fn from(error: std::io::Error) -> Self
    {
        PlotError::IoError(error)
    }
}

impl From<PolarsError> for PlotError
{
    fn from(error: PolarsError) -> Self
    {
        PlotError::PolarsError(error)
    }
}

impl From<hex::FromHexError> for PlotError
{
    fn from(error: hex::FromHexError) -> Self
    {
        PlotError::HexDecodeError(error)
    }
}
use clap::Parser;

#[allow(non_snake_case)]
#[derive(Debug, Parser)]
#[command(name = "plotxy", about = "Plots tabular data", version)]
struct Opt
{
    #[arg(value_name = "FILE")]
    /// optional file with on entry per line [default: STDIN]
    input: Option<PathBuf>,

    #[arg(long, short, default_value = "1")]
    /// column index to be used as X
    x: usize,

    #[arg(long, short, default_value = "0.3")]
    /// transparency channel
    alpha: f64,

    #[arg(long, short, default_value = "1E88E5")]
    /// default plot color
    plot_color: String,

    #[arg(long, short, default_value = "2")]
    /// column index to be used as Y
    y: usize,

    #[arg(long, short)]
    /// column index to be used as color facet
    color: Option<usize>,

    #[arg(long)]
    /// column index to be used as color gradient facet
    gradient: Option<usize>,

    // r"" makes it printable as escaped in default
    #[arg(short, long, default_value = r"\t")]
    /// column delimiter
    delimiter: String,

    #[arg(short = 'H', long)]
    /// input has header line (see also --skip)
    Header: bool,

    #[arg(long, short, default_value = "0")]
    /// skip lines before header
    skip: usize,

    #[arg(long, short)]
    /// plot logarithmic X-axis
    logx: bool,

    #[arg(long)]
    /// plot logarithmic Y-axis
    logy: bool,

    #[arg(long, default_value = "0.0")]
    /// minimum X dimension
    x_dim_min: f64,

    #[arg(long)]
    /// maximum X dimension
    x_dim_max: Option<f64>,

    #[arg(long, default_value = "0.0")]
    /// minimum Y dimension
    y_dim_min: f64,

    #[arg(long)]
    /// maximum Y dimension
    y_dim_max: Option<f64>,

    #[arg(long, short, value_name = "FILE")]
    /// file to save PNG plot to, default append .plotxy.png to input filename
    outfile: Option<PathBuf>,

    #[arg(long)]
    /// set output format to svg
    svg: bool,

    #[structopt(short, long)]
    /// title above the plot, default filename
    title: Option<String>,

    #[arg(long, default_value = "2560")]
    /// image width
    width: u32,

    #[arg(long, default_value = "1200")]
    /// image width
    height: u32,

    #[arg(long, default_value = "X")]
    /// x-axis label
    xdesc: String,

    #[arg(long, default_value = "Y")]
    /// y-axis label
    ydesc: String,

    #[arg(long, default_value = "70")]
    /// x-axis label area size
    xdesc_area: u32,

    #[arg(long, default_value = "100")]
    /// y-axis label area size
    ydesc_area: u32,

    #[arg(long, default_value = "sans-serif")]
    /// label font name
    label_font: String,

    #[arg(long, default_value = "24")]
    /// label font size
    label_font_size: u32,

    #[arg(long, default_value = "sans-serif")]
    /// axis description font name
    axis_desc_font: String,

    #[arg(long, default_value = "22")]
    /// axis description font size
    axis_desc_font_size: u32,

    #[arg(long, default_value = "sans-serif")]
    /// title font name
    title_font: String,

    #[arg(long, default_value = "24")]
    /// title font size
    title_font_size: u32,

    #[arg(long, default_value = "3")]
    /// point size, radius
    point_size: u32,

    #[arg(long, default_value = "circle")]
    /// plotting shape: circle, column
    shape: String,

    #[arg(long)]
    /// use SI number formatting for X-axis labels (K, M, G, etc.)
    si_format_x: bool,

    #[arg(long)]
    /// use SI number formatting for Y-axis labels (K, M, G, etc.)
    si_format_y: bool,
}

fn main() -> Result<(), PlotError>
{
    let mut opt = Opt::parse();

    let mut input: Box<dyn std::io::Read + 'static> = if let Some(path) = &opt.input
    {
        Box::new(std::fs::File::open(path)?)
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
                .ok_or_else(|| PlotError::InvalidData("Input path missing".to_string()))?
                .file_name()
                .ok_or_else(|| PlotError::InvalidData("Invalid input filename".to_string()))?
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

    let df = csv_read_options
        .into_reader_with_file_handle(Cursor::new(buf))
        .finish()?;

    plot_xy(&opt, df)
}

fn next_potence(x: f64) -> f64
{
    10f64.powf(((x.log10() * 10f64).ceil()) / 10.0)
}

fn format_si_number(value: f64) -> String
{
    let abs_value = value.abs();

    if abs_value >= 1e12
    {
        format!("{:.2}T", value / 1e12)
    }
    else if abs_value >= 1e9
    {
        format!("{:.2}G", value / 1e9)
    }
    else if abs_value >= 1e6
    {
        format!("{:.2}M", value / 1e6)
    }
    else if abs_value >= 1e3
    {
        format!("{:.2}K", value / 1e3)
    }
    else if abs_value >= 1.0
    {
        format!("{:.2}", value)
    }
    else if abs_value >= 1e-3
    {
        format!("{:.2}m", value * 1e3)
    }
    else if abs_value >= 1e-6
    {
        format!("{:.2}μ", value * 1e6)
    }
    else if abs_value >= 1e-9
    {
        format!("{:.2}n", value * 1e9)
    }
    else if abs_value >= 1e-12
    {
        format!("{:.2}p", value * 1e12)
    }
    else if abs_value == 0.0
    {
        "0".to_string()
    }
    else
    {
        format!("{:.2e}", value)
    }
}

fn plot_xy(opt: &Opt, df: DataFrame) -> Result<(), PlotError>
{
    let plot_filename = opt
        .outfile
        .as_ref()
        .ok_or_else(|| PlotError::InvalidData("Output file path missing".to_string()))?
        .to_str()
        .ok_or_else(|| PlotError::InvalidData("Invalid output file path".to_string()))?
        .to_string();

    println!("{}", plot_filename);

    let number_of_panels = 1;

    if opt.svg
    {
        plot_on_backend(
            opt,
            df,
            SVGBackend::new(&plot_filename, (opt.width, number_of_panels * opt.height)),
        )?;
    }
    else
    {
        plot_on_backend(
            opt,
            df,
            BitMapBackend::new(&plot_filename, (opt.width, number_of_panels * opt.height)),
        )?;
    }
    Ok(())
}

fn plot_on_backend<'a, B>(opt: &Opt, df: DataFrame, backend: B) -> Result<(), PlotError>
where
    B: DrawingBackend,
{
    let plot_filename = opt
        .outfile
        .as_ref()
        .ok_or_else(|| PlotError::InvalidData("Output file path missing".to_string()))?
        .to_str()
        .ok_or_else(|| PlotError::InvalidData("Invalid output file path".to_string()))?
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
    let x = if opt.x == 0
    {
        &idx
    }
    else
    {
        df.get_columns()
            .get(opt.x - 1)
            .ok_or_else(|| PlotError::InvalidColumn(format!("X column {} not found", opt.x)))?
            .as_series()
            .ok_or_else(|| PlotError::InvalidColumn("X column conversion failed".to_string()))?
    };
    let y = df
        .get_columns()
        .get(opt.y - 1)
        .ok_or_else(|| PlotError::InvalidColumn(format!("Y column {} not found", opt.y)))?
        .as_series()
        .ok_or_else(|| PlotError::InvalidColumn("Y column conversion failed".to_string()))?;
    let x_max: f64 = x
        .max()?
        .ok_or_else(|| PlotError::InvalidData("No data in X column".to_string()))?;
    let y_max: f64 = y
        .max()?
        .ok_or_else(|| PlotError::InvalidData("No data in Y column".to_string()))?;
    let _y_min: f64 = y
        .min()?
        .ok_or_else(|| PlotError::InvalidData("No data in Y column".to_string()))?;

    let xf64 = x.cast(&DataType::Float64)?;
    let yf64 = y.cast(&DataType::Float64)?;
    let xyc = make_xyc(&xf64, &yf64, &df, &opt)?;

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
            plot_shapes(&mut chart, shapes, &opt, x_max, y_max)?;
            Ok(())
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
            plot_shapes(&mut chart, shapes, &opt, x_max, y_max)?;
            Ok(())
        }
    }
}

/// Returns an iterator over x/y points and the color based on facet/gradient
fn make_xyc<'a, 'b>(
    x: &'a Series,
    y: &'b Series,
    df: &DataFrame,
    opt: &Opt,
) -> Result<
    Zip<
        Zip<
            Box<dyn PolarsIterator<Item = Option<f64>> + 'a>,
            Box<dyn PolarsIterator<Item = Option<f64>> + 'b>,
        >,
        std::vec::IntoIter<ShapeStyle>,
    >,
    PlotError,
>
{
    let plot_color = hex::decode(&opt.plot_color)?;
    let plot_plotters_color = RGBColor(plot_color[0], plot_color[1], plot_color[2]);
    let xy = x
        .f64()
        .map_err(|_| PlotError::InvalidData("X column is not numeric".to_string()))?
        .into_iter()
        .zip(
            y.f64()
                .map_err(|_| PlotError::InvalidData("Y column is not numeric".to_string()))?
                .into_iter(),
        );

    let color_iterator = if let Some(color_facet_index) = opt.color
    {
        let color_series = df.get_columns()
            .get(color_facet_index - 1)
            .ok_or_else(|| {
                PlotError::InvalidColumn(format!("Color column {} not found", color_facet_index))
            })?
            .as_series()
            .ok_or_else(|| PlotError::InvalidColumn("Color column conversion failed".to_string()))?;
        
        // Try to cast directly to Float64 first, fall back to String->Categorical->Float64 if needed
        let numeric_series = if color_series.dtype().is_primitive_numeric() {
            color_series.cast(&DataType::Float64)?
        } else {
            color_series
                .cast(&DataType::String)?
                .cast(&DataType::Categorical(None, CategoricalOrdering::Lexical))?
                .cast(&DataType::Float64)?
        };
        
        numeric_series
            .f64()
            .map_err(|_| PlotError::InvalidData("Color column is not numeric".to_string()))?
            .into_iter()
            .map(|c| ShapeStyle::from(Palette99::pick(c.unwrap_or(0.0) as usize)).filled())
            .collect()
    }
    else if let Some(color_gradient_index) = opt.gradient
    {
        get_gradient_color_iter(
            &opt,
            df.get_columns()
                .get(color_gradient_index - 1)
                .ok_or_else(|| {
                    PlotError::InvalidColumn(format!(
                        "Gradient column {} not found",
                        color_gradient_index
                    ))
                })?
                .as_series()
                .ok_or_else(|| {
                    PlotError::InvalidColumn("Gradient column conversion failed".to_string())
                })?,
        )?
    }
    else
    {
        (0..xy.len())
            .into_iter()
            .map(|_c| ShapeStyle::from(plot_plotters_color.mix(opt.alpha)).filled())
            .collect()
    };
    Ok(xy.zip(color_iterator))
}

// Macro to reduce duplication in mesh configuration
macro_rules! configure_and_draw_mesh {
    ($grid:expr, $opt:expr, $shapes:expr) => {{
        let mesh_result = match ($opt.si_format_x, $opt.si_format_y) {
            (true, true) => $grid
                .configure_mesh()
                .disable_x_mesh()
                .bold_line_style(WHITE.mix(0.3))
                .y_desc(&$opt.ydesc)
                .x_desc(&$opt.xdesc)
                .label_style(($opt.label_font.as_str(), $opt.label_font_size))
                .axis_desc_style(($opt.axis_desc_font.as_str(), $opt.axis_desc_font_size))
                .x_label_formatter(&|x| format_si_number(*x))
                .y_label_formatter(&|y| format_si_number(*y))
                .draw(),
            (true, false) => $grid
                .configure_mesh()
                .disable_x_mesh()
                .bold_line_style(WHITE.mix(0.3))
                .y_desc(&$opt.ydesc)
                .x_desc(&$opt.xdesc)
                .label_style(($opt.label_font.as_str(), $opt.label_font_size))
                .axis_desc_style(($opt.axis_desc_font.as_str(), $opt.axis_desc_font_size))
                .x_label_formatter(&|x| format_si_number(*x))
                .draw(),
            (false, true) => $grid
                .configure_mesh()
                .disable_x_mesh()
                .bold_line_style(WHITE.mix(0.3))
                .y_desc(&$opt.ydesc)
                .x_desc(&$opt.xdesc)
                .label_style(($opt.label_font.as_str(), $opt.label_font_size))
                .axis_desc_style(($opt.axis_desc_font.as_str(), $opt.axis_desc_font_size))
                .y_label_formatter(&|y| format_si_number(*y))
                .draw(),
            (false, false) => $grid
                .configure_mesh()
                .disable_x_mesh()
                .bold_line_style(WHITE.mix(0.3))
                .y_desc(&$opt.ydesc)
                .x_desc(&$opt.xdesc)
                .label_style(($opt.label_font.as_str(), $opt.label_font_size))
                .axis_desc_style(($opt.axis_desc_font.as_str(), $opt.axis_desc_font_size))
                .draw(),
        };
        mesh_result.map_err(|e| PlotError::InvalidData(format!("Draw error: {}", e)))?;
        $grid.draw_series($shapes)
            .map_err(|e| PlotError::InvalidData(format!("Backend Error: {}", e)))?;
    }};
}

fn plot_shapes<'a, 'b, DB, T>(
    chart: &mut ChartBuilder<'a, 'b, DB>,
    shapes: T,
    opt: &Opt,
    x_max: f64,
    y_max: f64,
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    T: IntoIterator,
    T::Item: Drawable<DB>,
    for<'d> &'d <T as IntoIterator>::Item: PointCollection<'d, (f64, f64)>,
{
    let x_dim_min = opt.x_dim_min;
    let y_dim_min = opt.y_dim_min;
    let x_dim_max = opt.x_dim_max.unwrap_or(next_potence(x_max as f64));
    let y_dim_max = opt.y_dim_max.unwrap_or(next_potence(y_max as f64));
    
    match (opt.logx, opt.logy) {
        (true, true) => {
            let mut grid = chart
                .build_cartesian_2d(
                    (x_dim_min..x_dim_max).log_scale(),
                    (y_dim_min..y_dim_max).log_scale(),
                )
                .map_err(|e| PlotError::InvalidData(format!("Grid creation error: {}", e)))?;
            configure_and_draw_mesh!(grid, opt, shapes);
        }
        (true, false) => {
            let mut grid = chart
                .build_cartesian_2d((x_dim_min..x_dim_max).log_scale(), y_dim_min..y_dim_max)
                .map_err(|e| PlotError::InvalidData(format!("Grid creation error: {}", e)))?;
            configure_and_draw_mesh!(grid, opt, shapes);
        }
        (false, true) => {
            let mut grid = chart
                .build_cartesian_2d(x_dim_min..x_dim_max, (y_dim_min..y_dim_max).log_scale())
                .map_err(|e| PlotError::InvalidData(format!("Grid creation error: {}", e)))?;
            configure_and_draw_mesh!(grid, opt, shapes);
        }
        (false, false) => {
            let mut grid = chart
                .build_cartesian_2d(x_dim_min..x_dim_max, y_dim_min..y_dim_max)
                .map_err(|e| PlotError::InvalidData(format!("Grid creation error: {}", e)))?;
            configure_and_draw_mesh!(grid, opt, shapes);
        }
    }
    Ok(())
}

fn get_gradient_color_iter(opt: &Opt, series: &Series) -> Result<Vec<ShapeStyle>, PlotError>
{
    let float_series = series.cast(&DataType::Float32)?;
    let values = float_series
        .f32()
        .map_err(|_| PlotError::InvalidData("Gradient column is not numeric".to_string()))?;
    let grad = colorgrad::GradientBuilder::new()
        .html_colors(&["yellow", "red"])
        .domain(&[
            values.min().ok_or_else(|| {
                PlotError::InvalidData("No minimum value in gradient column".to_string())
            })?,
            values.max().ok_or_else(|| {
                PlotError::InvalidData("No maximum value in gradient column".to_string())
            })?,
        ])
        .build::<colorgrad::LinearGradient>()
        .expect("prebuilt gradient should always work");

    let color_vec = values
        .into_iter()
        .map(|c| {
            ShapeStyle::from(
                rbgcolor_from_gradient(grad.at(c.unwrap_or(0.0) as f32).to_rgba8(), opt.alpha)
                    .filled(),
            )
        })
        .collect();
    Ok(color_vec)
}

fn rbgcolor_from_gradient(g: [u8; 4], alpha: f64) -> RGBAColor
{
    RGBAColor(g[0], g[1], g[2], alpha)
}
