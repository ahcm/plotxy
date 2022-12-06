# plotxy
Plot tabular data from the command line

# License
EUPL 1.2

# Usage
```
plotxy 0.1.0
Plots tabular data

USAGE:
    plotxy [FLAGS] [OPTIONS] [input]

FLAGS:
    -H, --Header     input has header line
    -h, --help       Prints help information
    -l, --logx       plot logarithmic Y-axis
    -l, --logy       plot logarithmic Y-axis
    -V, --version    Prints version information

OPTIONS:
    -a, --alpha <alpha>              transparancy channel [default: 0.3]
    -c, --color <color>              column index to be used as color facet
    -d, --delimiter <delimiter>      column delimiter [default: \t]
        --gradient <gradient>        column index to be used as color gradient facet
        --height <height>            image width [default: 1200]
    -o, --outfile <outfile>          file to save PNG plot to, default append .plotyy.png to input filename
    -p, --plot_color <plot_color>    default plot color [default: 1E88E5]
    -t, --title <title>              title above the plot, default filename
        --width <width>              image width [default: 2560]
    -x, --x <x>                      column index to be used as X [default: 1]
        --x_dim_max <x_dim_max>      maximum X dimension
        --x_dim_min <x_dim_min>      minimum X dimension [default: 0.0]
        --xdesc <xdesc>              x-axis label [default: X]
    -y, --y <y>                      column index to be used as Y [default: 2]
        --y_dim_max <y_dim_max>      maximum Y dimension
        --y_dim_min <y_dim_min>      minimum Y dimension [default: 0.0]
        --ydesc <ydesc>              y-axis label [default: Y]

ARGS:
    <input>    optional file with on entry per line [default: STDIN]
```
