FILE=$1
if [ -z "$FILE" ]
then
    echo "call me with a filename"
    exit 2
fi

SECONDS=`wc -l $FILE | cut -d ' ' -f 1`

cat << '__EOP' | sed "s/PLOT/$FILE/;s/SECONDS/$SECONDS/g;" | gnuplot > $FILE.png
    set terminal pngcairo size SECONDS*5+150,280 enhanced font 'Verdana,10'
    set yrange [0:100]
    set ytics nomirror
    set y2tics
    set format y "%.0f%%"
    set format y2 "%2.1fGB"

    set key on outside center bottom horizontal
    set xrange [0:SECONDS]

    y_rem = 0
    d(y) = ($0 == 0) ? (y_rem = y, 1/0) : (y2 = y_rem, y_rem = y, y_rem-y2)

    plot \
            "PLOT" using 1:($2*100/2037) title "done" with lines lc "dark-green" lw 2, \
            "PLOT" using 1:(100*d($7+$8)/$6) with lines t "cpu" lc "red" lw 2, \
            "PLOT" using 1:(d($9)/100/1000) title "faults" with lines axes x1y2 lc "pink" lw 2, \
            "PLOT" using 1:($5/1024/1024/1024) title "vsz" with lines axes x1y2 lc "blue" lw 2, \
            "PLOT" using 1:($4/1024/1024/1024) title "rss" with lines axes x1y2 lc "brown" lw 2, \
            "PLOT" using 1:($12/1024/1024/1024) title "alloc" with lines axes x1y2 lc "green" lw 2, \
            "PLOT" using 1:($13/1024/1024/1024) title "active" with lines axes x1y2 lc "orange" lw 2, \
            "PLOT" using 1:($16/1024/1024/1024) title "mapped" with lines axes x1y2 lc "purple" lw 2, \
            "PLOT" using 1:($18/10/1000/1000) title "read" with lines axes x1y1 lc "blue" lw 2 dt ".", \
            "PLOT" using 1:($19/10/1000/1000) title "aggre1" with lines axes x1y1 lc "purple" lw 2 dt ".", \
            "PLOT" using 1:($20/10/1000/1000) title "aggre2" with lines axes x1y1 lc "red" lw 2 dt "."
    
__EOP
