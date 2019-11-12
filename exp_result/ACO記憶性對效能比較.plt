# Unset all properties
unset title
unset key
unset xtics
unset ytics

# Setup output environment
set term postscript eps enhanced
set output "ACO記憶性對效能比較.eps"

# Graphic settings
set style data linespoints

set size 0.8,0.8

set xlabel "Memory" font "Arial,22" offset 0,-0.5
set ylabel "Computing time(microsecond)" font "Arial,22" offset -1,0

set logscale y

set xtics
set ytics
set key left top Left reverse nobox
set terminal postscript solid "Arial" 18

set grid y

set datafile missing "-"

# Plotting
plot [][] "ACO記憶性對效能比較.dat" \
	   u ($1):($2 / 1000) lt 1 pt 6 ps 2 t "ACO", \