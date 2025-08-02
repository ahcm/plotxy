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

## Version 0.2.2 and up on MacOS

rust-roolchain.toml shuld take care for that for now.
For other platforms you might want to delete it.

There is a SIMD change, that has not been reflected in a released version of pathfinder, an underlying font raster library.
Try to use a rust version before the SIMD change, e.g.:
```
cargo +nightly-2024-08-04 install --locked plotxy
````

# Usage
```
plotxy 0.3.0
Plots tabular data

Usage: plotxy [OPTIONS] [FILE]

Arguments:
  [FILE]  optional file with on entry per line [default: STDIN]

Options:
  -x, --x <X>
          column index to be used as X [default: 1]
  -a, --alpha <ALPHA>
          transparency channel [default: 0.3]
  -p, --plot-color <PLOT_COLOR>
          default plot color [default: 1E88E5]
  -y, --y <Y>
          column index to be used as Y [default: 2]
  -c, --color <COLOR>
          column index to be used as color facet
      --gradient <GRADIENT>
          column index to be used as color gradient facet
  -d, --delimiter <DELIMITER>
          column delimiter [default: \t]
  -H, --header
          input has header line (see also --skip)
  -s, --skip <SKIP>
          skip lines before header [default: 0]
  -l, --logx
          plot logarithmic X-axis
      --logy
          plot logarithmic Y-axis
      --x-dim-min <X_DIM_MIN>
          minimum X dimension [default: 0.0]
      --x-dim-max <X_DIM_MAX>
          maximum X dimension
      --y-dim-min <Y_DIM_MIN>
          minimum Y dimension [default: 0.0]
      --y-dim-max <Y_DIM_MAX>
          maximum Y dimension
  -o, --outfile <FILE>
          file to save PNG plot to, default append .plotxy.png to input filename
      --svg
          set output format to svg
  -t, --title <TITLE>
          title above the plot, default filename
      --width <WIDTH>
          image width [default: 2560]
      --height <HEIGHT>
          image width [default: 1200]
      --xdesc <XDESC>
          x-axis label [default: X]
      --ydesc <YDESC>
          y-axis label [default: Y]
      --xdesc-area <XDESC_AREA>
          x-axis label area size [default: 70]
      --ydesc-area <YDESC_AREA>
          y-axis label area size [default: 100]
      --label-font <LABEL_FONT>
          label font name [default: sans-serif]
      --label-font-size <LABEL_FONT_SIZE>
          label font size [default: 24]
      --axis-desc-font <AXIS_DESC_FONT>
          axis description font name [default: sans-serif]
      --axis-desc-font-size <AXIS_DESC_FONT_SIZE>
          axis description font size [default: 22]
      --title-font <TITLE_FONT>
          title font name [default: sans-serif]
      --title-font-size <TITLE_FONT_SIZE>
          title font size [default: 24]
      --point-size <POINT_SIZE>
          point size, radius [default: 3]
      --shape <SHAPE>
          plotting shape: circle, column [default: circle]
      --si-format-x
          use SI number formatting for X-axis labels (K, M, G, etc.)
      --si-format-y
          use SI number formatting for Y-axis labels (K, M, G, etc.)
  -h, --help
          Print help
  -V, --version
          Print version
```
