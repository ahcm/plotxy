use plotters::prelude::*;
use polars::prelude::*;
use std::error::Error;
use std::io::Cursor;
use std::path::PathBuf;
use structopt::StructOpt;

#[allow(non_snake_case)]
#[derive(Debug, StructOpt)]
#[structopt(
    name = "plotxy",
    about = "Plots tabular data",
    rename_all = "verbatim"
)]
struct Opt {
    #[structopt(parse(from_os_str))]
    /// optional file with on entry per line [default: STDIN]
    input: Option<PathBuf>,

    #[structopt(long, short, default_value = "1")]
    /// column index to be used as X
    x: usize,

    #[structopt(long, short, default_value="0.3")]
    /// transparancy channel
    alpha: f64,

    #[structopt(long, short, default_value="1E88E5")]
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
    /// input has header line
    header: bool,

    #[structopt(long, short)]
    /// plot logarithmic Y-axis
    logy: bool,

    #[structopt(parse(from_os_str), long, short)]
    /// file to save PNG plot to, default append .plotyy.png to input filename
    outfile: Option<PathBuf>,

    #[structopt(short, long)]
    /// title above the plot, default filename
    title: Option<String>,

    #[structopt(short, long, default_value = "1280x960")]
    /// the x and y size of the plot
    geometry: String,

    #[structopt(long, default_value = "X")]
    /// x-axis label
    xdesc: String,

    #[structopt(long, default_value = "Y")]
    /// y-axis label
    ydesc: String,
}

fn main() -> std::result::Result<(), Box<dyn Error>>
{
    let mut opt = Opt::from_args();

    let mut input: Box<dyn std::io::Read + 'static> =
        if let Some(path) = &opt.input {
            Box::new(std::fs::File::open(path).unwrap())
        } else {
            opt.input = Some(std::path::Path::new("STDIN").to_path_buf());
            Box::new(std::io::stdin())
        };

    if opt.outfile.is_none()
    {
        let mut outname = PathBuf::new();
        outname.set_file_name(format!("{}{}", opt.input.as_ref().unwrap().file_name().unwrap().to_string_lossy(), ".plotxy.png"));
        opt.outfile = Some(outname)
    }

    // accept escaped delimiters
    // could be expanded to aliases e.g. "TAB"
    let delimiter = match opt.delimiter.as_str() {
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
        .has_header(opt.header)
        .finish()
        .unwrap();
    plot_xy(&opt, df)
}

fn next_potence(x: f64) -> f64 {
    10f64.powf(((x.log10() * 10f64).ceil()) / 10.0)
}

fn plot_xy(opt: &Opt, df: DataFrame) -> std::result::Result<(), Box<dyn Error>> {
    let plot_filename = opt.outfile.as_ref().expect("Outfile missing").to_str().unwrap().to_string();
    println!("{}", plot_filename);

    let plot_color = hex::decode(&opt.plot_color).expect("Decoding failed");
    let plot_plotters_color = RGBColor(plot_color[0], plot_color[1], plot_color[2]);
    let number_of_panels = 1;
    let xdesc = &opt.xdesc;
    let ydesc = &opt.ydesc;
    let idx : Series = (0..df.height() as i64).collect();
    let x = if opt.x == 0 { &idx } else { &df[opt.x - 1] };
    let y = &df[opt.y - 1];
    let x_max: i64 = x.max().expect("x is non numerical? If file has a header use -h");
    let y_max: i64 = y.max().expect("y is non numerical? If file has a header use -h");
    let x_dim: i64 = next_potence(x_max as f64) as i64;
    let y_dim: i64 = next_potence(y_max as f64) as i64;
    let root = BitMapBackend::new(&plot_filename, (2560, number_of_panels as u32 * 1200)).into_drawing_area();
    let panels = root.split_evenly((number_of_panels as usize, 1));
    root.fill(&WHITE)?;
    root.titled(opt.title.as_ref().unwrap_or(&plot_filename), ("sans-serif", 20))?;
    let mut chart = ChartBuilder::on(&panels[0])
        .x_label_area_size(70u32)
        .y_label_area_size(100u32)
        .margin(26u32)
        //.caption(format!("{ref_name}:{ref_start}-{ref_len}"), ("sans-serif", 20u32))
        .build_cartesian_2d(
            //(0u64..x_dim).into_segmented(),
            0i64..x_dim,
            0i64..y_dim,
        )?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(WHITE.mix(0.3))
        .y_desc(ydesc)
        .x_desc(xdesc)
        .label_style(("sans-serif", 24u32))
        .axis_desc_style(("sans-serif", 22u32))
        .draw()?;

    let xy = x.i64()
                .expect("x")
                .into_iter()
                .zip(y.i64().expect("y").into_iter())
            //.zip(df[3].i64().expect("facet as i64").into_iter())
            //    .zip(std::iter::repeat(1).map(|c| Some(c)))
                ;

    let blue_iter : Series = std::iter::repeat(1i64).take(df.height()).collect();

    let color_iterator : Vec<ShapeStyle> =
        if let Some(color_facet_index) = opt.color
        {
            df[color_facet_index - 1].i64().expect("facet as i64").into_iter()
                    .map(|c| ShapeStyle::from(Palette99::pick(c.unwrap_or(0) as usize)).filled()).collect()
        }
        else if let Some(color_gradient_index) = opt.gradient
        {
            df[color_gradient_index - 1].i64().expect("facet as i64").into_iter()
                .map(|c| ShapeStyle::from(Palette100::pick(c.unwrap_or(0) as usize)).filled()).collect()
        }
        else
        {
            blue_iter.i64().expect("oops on blue iterator").into_iter()
                .map(|_c| ShapeStyle::from(plot_plotters_color.mix(opt.alpha)).filled()).collect()
        };

    let shapes = xy.zip(color_iterator) //.zip(color_iterator)
        .map(|((x, y), c)|
             match (x, y) {
                 (Some(xx), Some(yy)) =>
                   {
                     Circle::new( (xx, yy), 5, c)
                   }
                 _ => {
                     println!("NA value as 0 0");
                     Circle::new((0, 0), 5, c)
                 }
             });

    chart.draw_series(shapes).expect("Backend Error");
    Ok(())
}

