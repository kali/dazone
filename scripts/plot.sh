
function median {
    FILE=$1
    count=`grep '^0 ' $FILE | cut -f 2 -d ' ' | wc -l`
    head -n $(($count/2)) $FILE | cut -f 2 -d ' ' | tail -1
}

function best_for_hosts {
    grep -rh "2016" data | grep inner_time | grep -v " 0 inner" | ./exp.pl \
        | grep "type: *$2" \
        | grep "length:  *$1" \
        | sort -n -k 16n,16n -k 30,30n \
        | perl -ne 'm/.*hosts: *([0-9]+).*/; print unless $done{$1}; $done{$1}=1;'
}

function draw_scalability {
    NAME=$1
    best_for_hosts  8 $NAME > /tmp/k8
    best_for_hosts 10 $NAME > /tmp/k10
    best_for_hosts 12 $NAME > /tmp/k12
    max=`cat /tmp/k8 /tmp/k10 /tmp/k12 | perl -pe 's/ +/ /g' | cut -f 16 -d ' ' | sort -n | tail -1`
    cat <<- PLOT | gnuplot
        set terminal pngcairo size 800,400
        set yrange [0:]
        set y2range [0:]
        set xrange [0:$max+1]
        set xlabel "nodes"
        set ylabel "seconds"
        set y2label "cost efficiency"
        set output "2016-02-15-$NAME.time.png"
        set boxwidth 0.10
        set ytics nomirror
        set y2tics nomirror
        set style fill solid 0.6 noborder
        set key outside bottom center horiz
        plot  \
            "/tmp/k8"  using (\$16-0.20):(8.5*25/\$4/\$16/\$30) with boxes title "Query2A" axis x1y2 , \
            "/tmp/k10" using (\$16     ):(8.5*56/\$4/\$16/\$30) with boxes title "Query2B" axis x1y2, \
            "/tmp/k12" using (\$16+0.20):(8.5*79/\$4/\$16/\$30) with boxes title "Query2C" axis x1y2, \
            "/tmp/k8"  using 16:30 notitle with lines ls 1 lw 2, \
            "/tmp/k10" using 16:30 notitle with lines ls 2 lw 2, \
            "/tmp/k12" using 16:30 notitle with lines ls 3 lw 2;
PLOT
}

draw_scalability c3.2xlarge
draw_scalability c3.8xlarge
draw_scalability m3.2xlarge
draw_scalability m3.xlarge
