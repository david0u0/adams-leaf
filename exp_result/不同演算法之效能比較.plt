# Unset all properties
unset title
unset key
unset xtics
unset ytics

# Setup output environment
set term postscript eps enhanced
set output "不同演算法之效能比較1.eps"

# Graphic settings
set style data linespoints

set size 0.8,0.8

set xlabel "Number of new flows" font "Arial,22" offset 0,-0.5
set ylabel "Computing time (millisecond)" font "Arial,22" offset -1,0

set logscale y

set xtics
set ytics
set key left top Left reverse nobox
set terminal postscript solid "Arial" 18

set grid y

set datafile missing "-"

# Plotting
plot [][] "不同演算法之效能比較1.dat" \
	   u ($1):($2 / 1000) lt 1 pt 6 ps 2 t "RO", \
	"" u ($1):($3 / 1000) lt 2 pt 7 ps 2 t "ACO", \
	"" u ($1):($4 / 1000) lt 1 pt 8 ps 2 t "ACO w/o reroute", \

set output "不同演算法之效能比較2.eps"
# Plotting
plot [][] "不同演算法之效能比較2.dat" \
	   u ($1):($2 / 1000) lt 1 pt 6 ps 2 t "RO", \
	"" u ($1):($3 / 1000) lt 2 pt 7 ps 2 t "ACO", \
	"" u ($1):($4 / 1000) lt 1 pt 8 ps 2 t "ACO w/o reroute", \