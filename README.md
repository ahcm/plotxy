# plotxy
Plot tabular data from the command line

# License
EUPL 1.2

# Install

```
cargo install plotxy
```

In general if install fails, try with --locked.

```
cargo install --locked plotxy
```

## Version 0.2.2 on MacOS

There is a SIMD change, that has not been reflected in a released version of pathfinder, an underlying font raster library.
Try to use a rust version before the SIMD change, e.g.:
```
cargo +nightly-2024-08-04 install --locked plotxy
````

# Usage
```
plotxy 0.2.2
Plots tabular data

USAGE:
    plotxy.bak [FLAGS] [OPTIONS] [input]

FLAGS:
    -H, --Header     input has header line (see also --skip)
    -h, --help       Prints help information
    -l, --logx       plot logarithmic Y-axis
    -l, --logy       plot logarithmic Y-axis
        --svg        set output format to svg
    -V, --version    Prints version information

OPTIONS:
    -a, --alpha <alpha>                                transparancy channel [default: 0.3]
        --axis_desc_font <axis_desc_font>              axis description font name [default: sans-serif]
        --axis_desc_font_size <axis_desc_font_size>    axis description font size [default: 22]
    -c, --color <color>                                column index to be used as color facet
    -d, --delimiter <delimiter>                        column delimiter [default: \t]
        --gradient <gradient>                          column index to be used as color gradient facet
        --height <height>                              image width [default: 1200]
        --label_font <label_font>                      label font name [default: sans-serif]
        --label_font_size <label_font_size>            label font size [default: 24]
    -o, --outfile <outfile>
            file to save PNG plot to, default append .plotxy.png to input filename

    -p, --plot_color <plot_color>                      default plot color [default: 1E88E5]
        --point_size <point_size>                      point size, radius [default: 3]
        --shape <shape>                                plotting shape: circle, column [default: circle]
    -s, --skip <skip>                                  skip lines before header [default: 0]
    -t, --title <title>                                title above the plot, default filename
        --title_font <title_font>                      title font name [default: sans-serif]
        --title_font_size <title_font_size>            title font size [default: 24]
        --width <width>                                image width [default: 2560]
    -x, --x <x>                                        column index to be used as X [default: 1]
        --x_dim_max <x_dim_max>                        maximum X dimension
        --x_dim_min <x_dim_min>                        minimum X dimension [default: 0.0]
        --xdesc <xdesc>                                x-axis label [default: X]
        --xdesc_area <xdesc_area>                      x-axis label area size [default: 70]
    -y, --y <y>                                        column index to be used as Y [default: 2]
        --y_dim_max <y_dim_max>                        maximum Y dimension
        --y_dim_min <y_dim_min>                        minimum Y dimension [default: 0.0]
        --ydesc <ydesc>                                y-axis label [default: Y]
        --ydesc_area <ydesc_area>                      y-axis label area size [default: 100]

ARGS:
    <input>    optional file with on entry per line [default: STDIN]
```
