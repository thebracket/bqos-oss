var masterGraphList = [];

function toLocalTime(d) {
    let tzOffset = new Date().getTimezoneOffset() / 60;
    let date = new Date(d);
    date.setHours(date.getHours() - tzOffset);
    //let providedOffset = date.getTimezoneOffset() / 60;
    //console.log(d, date, providedOffset, tzOffset, (tzOffset - providedOffset), date.toISOString());
    //if (providedOffset != tzOffset) {
        //date.setHours(date.getHours() - (tzOffset - providedOffset));
        //console.log(d, date, tzOffset, providedOffset);
    //}
    let result = date.toISOString();
    //console.log(result);
    return result;
}

class TimeGraph {
    constructor(div, site, period, register = true) {
        this.div = div;
        this.site = site;
        this.setPeriod(period);
        this.graph_div = this.div + "_g";
        let graphDiv = $("<div style='width: 100%; height: 100%;'><img src='/spinner.gif' /></div>");
        let webDiv = $("#" + this.div);;
        webDiv.append(graphDiv);
        graphDiv.attr('id', this.graph_div);
        this.button_div = this.div + "_gb";
        let btnDiv = $("<div style='position: absolute; right: 8px; top: 8px; z-index: 500;'></div>");
        webDiv.prepend(btnDiv);
        btnDiv.attr('id', this.button_div);
        this.buildButton("15m", "15m");
        this.buildButton("1h", "1h");
        this.buildButton("6h", "6h");
        this.buildButton("12h", "12h");
        this.buildButton("24h", "24h");
        this.buildButton("7d", "7d");
        this.buildButton("1m", "1m");
        this.buildButton("3m", "3m");
        this.buildButton("1y", "1y");
        this.buildZoomButton();
        if (register) {
            masterGraphList.push(this);
        }
    }

    buildButton(name, period) {
        let btn = $("<button style='font-size: 8pt; background-color: navy; color: white;'>" + name + "</button>");
        $("#" + this.button_div).append(btn);
        btn.click(() => {
            for (let i=0; i<masterGraphList.length; i++) {
                $("#" + masterGraphList[i].graph_div).html("<img src='/spinner.gif' />");
                masterGraphList[i].setPeriod(period);
                masterGraphList[i].render();
            }
        });
        //let itsme = this;
        //btn.click(() => { itsme.setPeriod(period); itsme.render(); });
    }

    buildZoomButton() {
        let btn = $("<button style='font-size: 8pt; margin-left: 4px;' data-bs-toggle='modal' data-bs-target='#exampleModal'>🔍</button>");
        $("#" + this.button_div).append(btn);
        let itsme = this;
        btn.click(() => {
            setTimeout(() => {
                $("#" + itsme.graph_div).html("<img src='/spinner.gif' />");
                let newGraph = itsme;
                newGraph.graph_div = "modalBody";
                newGraph.render();
            }, 100);
        });
    }

    setPeriod(period) {
        switch (period) {
            case "5m": { this.range = "start: -5m"; this.period = "1m"; break; }
            case "15m": { this.range = "start: -15m"; this.period = "1m"; break; }
            case "30m": { this.range = "start: -30m"; this.period = "2m"; break; }
            case "1h": { this.range = "start: -1h"; this.period = "150s"; break; }
            case "2h": { this.range = "start: -2h"; this.period = "3m"; break; }
            case "6h": { this.range = "start: -6h"; this.period = "5m"; break; }
            case "12h": { this.range = "start: -12h"; this.period = "10m"; break; }
            case "24h": { this.range = "start: -24h"; this.period = "1h"; break; }
            case "7d": { this.range = "start: -7d"; this.period = "4h"; break; }
            case "1m": { this.range = "start: -30d"; this.period = "8h"; break; }
            case "3m": { this.range = "start: -90d"; this.period = "12h"; break; }
            case "1y": { this.range = "start: -365d"; this.period = "24h"; break; }
            default: { this.range = "start: -1h"; this.period = "1m"; }
        }
    }

    render() {
        document.getElementById(this.graph_div).on('plotly_relayout', (eventdata) => {
            let start = new Date(eventdata['xaxis.range[0]']);
            let end = new Date(eventdata['xaxis.range[1]']);
            let tzOffset = new Date().getTimezoneOffset() / 60;
            //start.setHours(start.getHours() - tzOffset);
            //end.setHours(end.getHours() - tzOffset);
            start = start.toISOString();
            end = end.toISOString();
            this.range = "start: " + start + ", stop: " + end;
            this.period = "1m";
            this.render();
        });
    }
}

class LatencyGraph extends TimeGraph {
    constructor(div, site, period) {
        super(div, site, period);
    }

    render() {
        let range = encodeURIComponent(this.range);
        let period = encodeURIComponent(this.period);
        $.get("/query/latency_site/" + this.site + "/" + range + "/" + period, (data) => {
            console.log(data.length);
            let latency = {
                x: [],
                y: [],
                type: 'scatter',
                name: "Average",
                mode: "lines",
            };
            let latencyMin = {
                x: [],
                y: [],
                type: 'scatter',
                name: "Min",
                fill: "tonexty", 
                fillcolor: "rgba(128,128,243,0.2)", 
                line: {color: "transparent"},
                mode: "lines"
            };
            let latencyMax = {
                x: [],
                y: [],
                type: 'scatter',
                name: "Max",
                mode: "lines",
                fill: "tonexty", 
                fillcolor: "rgba(128,128,243,0.2)", 
                line: {color: "transparent"}
            };
            for (let i = 0; i < data.length; i++) {
                latency.x.push(toLocalTime(data[i].date));
                latency.y.push(data[i].avg);
                latencyMin.x.push(toLocalTime(data[i].date));
                latencyMin.y.push(data[i].min);
                latencyMax.x.push(toLocalTime(data[i].date));
                latencyMax.y.push(data[i].max);
            }
            let bw = document.getElementById(this.graph_div);
            let layout = {
                showlegend: false,
                autosize: true,
                margin: {
                    l: 0,
                    r: 0,
                    b: 0,
                    t: 0,
                    pad: 4
                },
                yaxis: {
                    title: 'ms',
                    automargin: true,
                    titlefont: { size: 10 },
                },
                xaxis: {
                    title: "Date/Time",
                    automargin: true,
                    titlefont: { size: 10 },
                }
            }
            let buttonsToRemove = ["zoom2d", "pan2d", "select2d", "lasso2d", "zoomIn2d", "zoomOut2d", "autoScale2d", "resetScale2d"];
            Plotly.newPlot(bw, [latency, latencyMax, latencyMin], layout, { responsive: true, modeBarButtonsToRemove: buttonsToRemove });
            super.render();
        });
    }
}

class FrequencyGraph extends TimeGraph {
    constructor(div, site, period) {
        super(div, site, period);
    }

    render() {
        let range = encodeURIComponent(this.range);
        let period = encodeURIComponent(this.period);
        $.get("/query/ap_frequency/" + this.site + "/" + range + "/" + period, (data) => {
            console.log(data.length);
            let latency = {
                x: [],
                y: [],
                type: 'scatter'
            };
            for (let i = 0; i < data.length; i++) {
                latency.x.push(toLocalTime(data[i].date));
                latency.y.push(data[i].avg);
            }
            let bw = document.getElementById(this.graph_div);
            let layout = {
                showlegend: false,
                autosize: true,
                margin: {
                    l: 0,
                    r: 0,
                    b: 0,
                    t: 0,
                    pad: 4
                },
                yaxis: {
                    title: 'Mhz',
                    automargin: true,
                    titlefont: { size: 10 },
                },
                xaxis: {
                    title: "Date/Time",
                    automargin: true,
                    titlefont: { size: 10 },
                }
            }
            let buttonsToRemove = ["zoom2d", "pan2d", "select2d", "lasso2d", "zoomIn2d", "zoomOut2d", "autoScale2d", "resetScale2d"];
            Plotly.newPlot(bw, [latency], layout, { responsive: true, modeBarButtonsToRemove: buttonsToRemove });
            super.render();
        });
    }
}

class NoiseGraph extends TimeGraph {
    constructor(div, site, period) {
        super(div, site, period);
    }

    render() {
        let range = encodeURIComponent(this.range);
        let period = encodeURIComponent(this.period);
        $.get("/query/ap_noise/" + this.site + "/" + range + "/" + period, (data) => {
            console.log(data.length);
            let latency = {
                x: [],
                y: [],
                type: 'scatter'
            };
            for (let i = 0; i < data.length; i++) {
                latency.x.push(toLocalTime(data[i].date));
                latency.y.push(data[i].avg);
            }
            let bw = document.getElementById(this.graph_div);
            let layout = {
                showlegend: false,
                autosize: true,
                margin: {
                    l: 0,
                    r: 0,
                    b: 0,
                    t: 0,
                    pad: 4
                },
                yaxis: {
                    title: 'dB',
                    automargin: true,
                    titlefont: { size: 10 },
                },
                xaxis: {
                    title: "Date/Time",
                    automargin: true,
                    titlefont: { size: 10 },
                }
            }
            let buttonsToRemove = ["zoom2d", "pan2d", "select2d", "lasso2d", "zoomIn2d", "zoomOut2d", "autoScale2d", "resetScale2d"];
            Plotly.newPlot(bw, [latency], layout, { responsive: true, modeBarButtonsToRemove: buttonsToRemove });
            super.render();
        });
    }
}

class SignalGraph extends TimeGraph {
    constructor(div, site, period) {
        super(div, site, period);
    }

    render() {
        let range = encodeURIComponent(this.range);
        let period = encodeURIComponent(this.period);
        $.get("/query/signal/" + this.site + "/" + range + "/" + period, (data) => {
            console.log(data.length);
            let latency = {
                x: [],
                y: [],
                type: 'scatter'
            };
            for (let i = 0; i < data.length; i++) {
                latency.x.push(toLocalTime(data[i].date));
                latency.y.push(data[i].avg);
            }
            let bw = document.getElementById(this.graph_div);
            let layout = {
                showlegend: false,
                autosize: true,
                margin: {
                    l: 0,
                    r: 0,
                    b: 0,
                    t: 0,
                    pad: 4
                },
                yaxis: {
                    title: 'dB',
                    automargin: true,
                    titlefont: { size: 10 },
                },
                xaxis: {
                    title: "Date/Time",
                    automargin: true,
                    titlefont: { size: 10 },
                }
            }
            let buttonsToRemove = ["zoom2d", "pan2d", "select2d", "lasso2d", "zoomIn2d", "zoomOut2d", "autoScale2d", "resetScale2d"];
            Plotly.newPlot(bw, [latency], layout, { responsive: true, modeBarButtonsToRemove: buttonsToRemove });
            super.render();
        });
    }
}

class BandwidthGraph extends TimeGraph {
    constructor(div, site, period) {
        super(div, site, period);
    }

    render() {
        let range = encodeURIComponent(this.range);
        let period = encodeURIComponent(this.period);
        $.get("/query/site_bandwidth/" + this.site + "/" + range + "/" + period, (data) => {
            //console.log(data);
            let up = {
                x: [],
                y: [],
                type: 'scatter',
                name: 'Upload',
                fill: 'tozeroy',
                fillcolor: '#0000ff',
            };
            let down = {
                x: [],
                y: [],
                type: 'scatter',
                name: 'Download',
                fill: 'tozeroy',
            };
            for (let i = 0; i < data.length; i++) {
                up.x.push(toLocalTime(data[i].time));
                up.y.push(data[i].up);
                down.x.push(toLocalTime(data[i].time));
                down.y.push(data[i].down);
            }
            let bw = document.getElementById(this.graph_div);
            let layout = {
                showlegend: false,
                autosize: true,
                margin: {
                    l: 0,
                    r: 0,
                    b: 0,
                    t: 0,
                    pad: 4
                },
                yaxis: {
                    title: 'Mbps',
                    tickmode: 'array',
                    automargin: true,
                    titlefont: { size: 10 },
                },
                xaxis: {
                    title: "Date/Time",
                    automargin: true,
                    titlefont: { size: 10 },
                }
            }
            let buttonsToRemove = ["zoom2d", "pan2d", "select2d", "lasso2d", "zoomIn2d", "zoomOut2d", "autoScale2d", "resetScale2d"];
            Plotly.newPlot(bw, [up, down], layout, { responsive: true, modeBarButtonsToRemove: buttonsToRemove });
            super.render();
        });
    }
}

class DropsGraph extends TimeGraph {
    constructor(div, site, period) {
        super(div, site, period);
    }

    render() {
        let range = encodeURIComponent(this.range);
        let period = encodeURIComponent(this.period);
        $.get("/query/site_drops/" + this.site + "/" + range + "/" + period, (data) => {
            //console.log(data);
            let up = {
                x: [],
                y: [],
                type: 'scatter',
                name: 'Up Drops',
                fill: 'tozeroy',
                fillcolor: '#0000ff',
            };
            let down = {
                x: [],
                y: [],
                type: 'scatter',
                name: 'Down Drops',
                fill: 'tozeroy',
            };
            for (let i = 0; i < data.length; i++) {
                up.x.push(toLocalTime(data[i].time));
                up.y.push(data[i].up);
                down.x.push(toLocalTime(data[i].time));
                down.y.push(data[i].down);
            }
            let bw = document.getElementById(this.graph_div);
            let layout = {
                showlegend: false,
                autosize: true,
                margin: {
                    l: 0,
                    r: 0,
                    b: 0,
                    t: 0,
                    pad: 4
                },
                yaxis: {
                    title: 'Drops',
                    tickmode: 'array',
                    automargin: true,
                    titlefont: { size: 10 },
                },
                xaxis: {
                    title: "Date/Time",
                    automargin: true,
                    titlefont: { size: 10 },
                }
            }
            let buttonsToRemove = ["zoom2d", "pan2d", "select2d", "lasso2d", "zoomIn2d", "zoomOut2d", "autoScale2d", "resetScale2d"];
            Plotly.newPlot(bw, [up, down], layout, { responsive: true, modeBarButtonsToRemove: buttonsToRemove });
            super.render();
        });
    }
}

class BandwidthGraphWithLimits extends TimeGraph {
    constructor(div, site, period, maxDown, maxUp) {
        super(div, site, period);
        this.maxDown = maxDown;
        this.maxUp = maxUp;
    }

    render() {
        let range = encodeURIComponent(this.range);
        let period = encodeURIComponent(this.period);
        $.get("/query/site_bandwidth/" + this.site + "/" + range + "/" + period, (data) => {
            //console.log(data);
            let up = {
                x: [],
                y: [],
                type: 'scatter',
                name: 'Upload',
                fill: 'tozeroy'
            };
            let down = {
                x: [],
                y: [],
                type: 'scatter',
                name: 'Download',
                fill: 'tozeroy'
            };
            let upMax = {
                x: [],
                y: [],
                type: 'scatter',
                name: 'Limit ▲',
                line: {
                    color: '#ffdddd'
                },
            };
            let downMax = {
                x: [],
                y: [],
                type: 'scatter',
                name: 'Limit ▼',
                line: {
                    color: '#ddffdd'
                }
            };
            for (let i = 0; i < data.length; i++) {
                up.x.push(toLocalTime(data[i].time));
                up.y.push(0 - data[i].up);
                down.x.push(toLocalTime(data[i].time));
                down.y.push(data[i].down);
                downMax.y.push(this.maxDown);
                upMax.y.push(0 - this.maxUp);
                downMax.x.push(toLocalTime(data[i].time));
                upMax.x.push(toLocalTime(data[i].time));
            }
            let bw = document.getElementById(this.graph_div);
            let layout = {
                showLegend: false,
                autosize: true,
                margin: {
                    l: 0,
                    r: 0,
                    b: 0,
                    t: 0,
                    pad: 4
                },
                yaxis: {
                    title: 'Mbps',
                    tickmode: 'array',
                    automargin: true,
                    titlefont: { size: 10 },
                },
                xaxis: {
                    title: "Date/Time",
                    automargin: true,
                    titlefont: { size: 10 },
                }
            }
            let buttonsToRemove = ["zoom2d", "pan2d", "select2d", "lasso2d", "zoomIn2d", "zoomOut2d", "autoScale2d", "resetScale2d"];
            Plotly.newPlot(bw, [up, down, upMax, downMax], layout, { responsive: true, modeBarButtonsToRemove: buttonsToRemove });
            super.render();
        });
    }
}

class DownwardsFunnelGraph extends TimeGraph {
    constructor(div, site, period) {
        super(div, site, period);
    }

    render() {
        let bw = document.getElementById(this.graph_div);
        let data = [];
        for (let i = 0; i < this.history.sites.length; i++) {
            let trace = {
                x: [],
                y: [],
                //fill: 'tozeroy',
                //type: 'scatter',
                name: this.history.sites[i][0],
                stackgroup: 'one'
            };
            for (let j = 0; j < this.history.sites[i][1].length; j++) {
                let d = this.history.sites[i][1][j];
                trace.x.push(toLocalTime(d.time));
                trace.y.push(d.down);
            }
            data.push(trace);
        }

        let layout = {
            showLegend: false,
            autosize: true,
            margin: {
                l: 0,
                r: 0,
                b: 0,
                t: 0,
                pad: 4
            },
            yaxis: {
                title: 'Mbps',
                tickmode: 'array',
                automargin: true,
                titlefont: { size: 10 },
            },
            xaxis: {
                title: "Date/Time",
                automargin: true,
                titlefont: { size: 10 },
            }
        }

        let buttonsToRemove = ["zoom2d", "pan2d", "select2d", "lasso2d", "zoomIn2d", "zoomOut2d", "autoScale2d", "resetScale2d"];
        Plotly.newPlot(bw, data, layout, { responsive: true, modeBarButtonsToRemove: buttonsToRemove });
        super.render();
    }
}

class UpwardsFunnelGraph extends TimeGraph {
    constructor(div, site, period) {
        super(div, site, period);
    }

    render() {
        let bw = document.getElementById(this.graph_div);
        //console.log(history);
        let data = [];
        for (let i = 0; i < this.history.sites.length; i++) {
            let trace = {
                x: [],
                y: [],
                //fill: 'tozeroy',
                //type: 'scatter',
                name: this.history.sites[i][0],
                stackgroup: 'one'
            };
            for (let j = 0; j < this.history.sites[i][1].length; j++) {
                let d = this.history.sites[i][1][j];
                trace.x.push(toLocalTime(d.time));
                trace.y.push(d.up);
            }
            data.push(trace);
        }

        let layout = {
            showLegend: false,
            autosize: true,
            margin: {
                l: 0,
                r: 0,
                b: 0,
                t: 0,
                pad: 4
            },
            yaxis: {
                title: 'Mbps',
                tickmode: 'array',
                automargin: true,
                titlefont: { size: 10 },
            },
            xaxis: {
                title: "Date/Time",
                automargin: true,
                titlefont: { size: 10 },
            }
        }

        let buttonsToRemove = ["zoom2d", "pan2d", "select2d", "lasso2d", "zoomIn2d", "zoomOut2d", "autoScale2d", "resetScale2d"];
        Plotly.newPlot(bw, data, layout, { responsive: true, modeBarButtonsToRemove: buttonsToRemove });
        super.render();
    }
}

class DownwardsFunnelSitesGraph extends TimeGraph {
    constructor(div, site, period) {
        super(div, site, period);
    }

    render() {
        let bw = document.getElementById(this.graph_div);
        //console.log(history);
        let data = [];
        for (let i = 0; i < this.history.sites.length; i++) {
            let trace = {
                x: [],
                y: [],
                //fill: 'tozeroy',
                //type: 'scatter',
                name: this.history.sites[i][0],
                stackgroup: 'one'
            };
            for (let j = 0; j < this.history.sites[i][1].length; j++) {
                let d = this.history.sites[i][1][j];
                trace.x.push(toLocalTime(d.time));
                trace.y.push(d.down);
            }
            data.push(trace);
        }

        let layout = {
            showLegend: false,
            autosize: true,
            margin: {
                l: 0,
                r: 0,
                b: 0,
                t: 0,
                pad: 4
            },
            yaxis: {
                title: 'Mbps',
                tickmode: 'array',
                automargin: true,
                titlefont: { size: 10 },
            },
            xaxis: {
                title: "Date/Time",
                automargin: true,
                titlefont: { size: 10 },
            }
        }

        let buttonsToRemove = ["zoom2d", "pan2d", "select2d", "lasso2d", "zoomIn2d", "zoomOut2d", "autoScale2d", "resetScale2d"];
        Plotly.newPlot(bw, data, layout, { responsive: true, modeBarButtonsToRemove: buttonsToRemove });
        super.render();
    }
}

class UpwardsFunnelSitesGraph extends TimeGraph {
    constructor(div, site, period) {
        super(div, site, period);
    }

    render() {
        let bw = document.getElementById(this.graph_div);
        //console.log(history);
        let data = [];
        for (let i = 0; i < this.history.sites.length; i++) {
            let trace = {
                x: [],
                y: [],
                //fill: 'tozeroy',
                //type: 'scatter',
                name: this.history.sites[i][0],
                stackgroup: 'one'
            };
            for (let j = 0; j < this.history.sites[i][1].length; j++) {
                let d = this.history.sites[i][1][j];
                trace.x.push(toLocalTime(d.time));
                trace.y.push(d.up);
            }
            data.push(trace);
        }

        let layout = {
            showLegend: false,
            autosize: true,
            margin: {
                l: 0,
                r: 0,
                b: 0,
                t: 0,
                pad: 4
            },
            yaxis: {
                title: 'Mbps',
                tickmode: 'array',
                automargin: true,
                titlefont: { size: 10 },
            },
            xaxis: {
                title: "Date/Time",
                automargin: true,
                titlefont: { size: 10 },
            }
        }

        let buttonsToRemove = ["zoom2d", "pan2d", "select2d", "lasso2d", "zoomIn2d", "zoomOut2d", "autoScale2d", "resetScale2d"];
        Plotly.newPlot(bw, data, layout, { responsive: true, modeBarButtonsToRemove: buttonsToRemove });
        super.render();
    }
}

class FunnelGraphs {
    constructor(downDiv, upDiv, node, period, useSiteGraphs) {
        this.useSiteGraphs = useSiteGraphs;
        this.downDiv = downDiv;
        this.upDiv = upDiv;
        this.node = node;
        this.period = period;
        this.downFunnel = new DownwardsFunnelGraph(downDiv, node, period, false);
        this.upFunnel = new UpwardsFunnelGraph(upDiv, node, period, false);
        masterGraphList.push(this);
    }

    setPeriod(period) {
        this.downFunnel.setPeriod(period);
        this.upFunnel.setPeriod(period);
    }

    render() {
        let range = encodeURIComponent(this.downFunnel.range);
        let period = encodeURIComponent(this.downFunnel.period);

        if (this.useSiteGraphs) {
            $.get("/query/site_funnel/" + this.node + "/" + range + "/" + period, (history) => {
                this.downFunnel.history = history;
                this.upFunnel.history = history;
                this.downFunnel.render();
                this.upFunnel.render();
            });
        } else {
            $.get("/query/site_funnel_sites/" + this.node + "/" + range + "/" + period, (history) => {
                this.downFunnel.history = history;
                this.upFunnel.history = history;
                this.downFunnel.render();
                this.upFunnel.render();
            });
        }
    }
}

function updateCpu(div) {
    $.get("/query/cpu_load", (usage) => {
        let data = [
            {
                x: [],
                y: [],
                type: 'bar'
            }
        ];
        for (let i = 0; i < usage.length; i++) {
            data[0].x.push(i);
            data[0].y.push(usage[i].usage);
        }
        let cpuLoad = document.getElementById(div);
        let layout = {
            showLegend: true,
            autosize: true,
            margin: {
                l: 0,
                r: 0,
                b: 0,
                t: 0,
                pad: 4
            },
            yaxis: {
                title: '% Usage',
                tickmode: 'array',
                automargin: true,
                titlefont: { size: 10 },
            },
        }
        Plotly.newPlot(cpuLoad, data, layout, { responsive: true });
    });
}

function updateMemory(div) {
    $.get("/query/ram_use", (usage) => {
        let usageN = Number(usage[0].usage);
        let data = [
            {
                values: [usageN, 1.0 - usageN],
                labels: ["Used", "Free"],
                type: 'pie'
            }
        ];
        let cpuLoad = document.getElementById(div);
        let layout = {
            showLegend: true,
            autosize: true,
            margin: {
                l: 0,
                r: 0,
                b: 0,
                t: 0,
                pad: 4
            },
            yaxis: {
                title: '% Usage',
                tickmode: 'array',
                automargin: true,
                titlefont: { size: 10 },
            },
        }
        Plotly.newPlot(cpuLoad, data, layout, { responsive: true });
    });
}

function updateSwap(div) {
    $.get("/query/swap_use", (usage) => {
        let usageN = Number(usage[0].usage);
        let data = [
            {
                values: [usageN, 1.0 - usageN],
                labels: ["Used", "Free"],
                type: 'pie'
            }
        ];
        let cpuLoad = document.getElementById(div);
        let layout = {
            showLegend: true,
            autosize: true,
            margin: {
                l: 0,
                r: 0,
                b: 0,
                t: 0,
                pad: 4
            },
            yaxis: {
                title: '% Usage',
                tickmode: 'array',
                automargin: true,
                titlefont: { size: 10 },
            },
        }
        Plotly.newPlot(cpuLoad, data, layout, { responsive: true });
    });
}

function doSearch() {
    let term = $("#SearchBox").val();
    $.post("/query/search", term, (result) => {
        let html = "";
        for (let i = 0; i < result.length; i++) {
            switch (result[i].rtype) {
                case "ap" : html += "<a style='margin-right: 12px' href='/access_point/" + result[i].id + "'>" + result[i].name + " (" + result[i].rtype + ")</a>"; break;
                case "client" : html += "<a style='margin-right: 12px' href='/client/" + result[i].id + "'>" + result[i].name + " (" + result[i].rtype + ")</a>"; break;
                default: html += "<a style='margin-right: 12px' href='/site/" + result[i].id + "'>" + result[i].name + " (" + result[i].rtype + ")</a>"; break;
            }
        }
        $("#SearchResult").html(html);
    });
}

function setupSearch() {
    $("#SearchBox").change(doSearch);
    $("#SearchBox").keyup(doSearch);
}

function selectBreadcrumbChild() {
    let target = $("#childList").val();
    if (target != "") {
        location.href=target;
    }
}

function setupBreadcrumbs(siteId) {
    $.get("/query/site_crumbs/" + siteId, (crumbs) => {
        let crumbsR = crumbs.reverse();
        let html = "<nav aria-label='breadcrumb'><ol class='breadcrumb'>";
        for (let i=0; i<crumbsR.length-1; i++) {
            switch (crumbsR[i].level_type) {
                case "ap" : html += "<li class='breadcrumb-item'><a href='/access_point/" + crumbsR[i].id + "'>" + crumbsR[i].name + "</a></li>"; break;
                case "client" : html += "<li class='breadcrumb-item'><a href='/client/" + crumbsR[i].id + "'>" + crumbsR[i].name + "</a></li>"; break;
                default : html += "<li class='breadcrumb-item'><a href='/site/" + crumbsR[i].id + "'>" + crumbsR[i].name + "</a></li>"; break;
            }
        }
        let lastCrumb = crumbsR[crumbsR.length-1];
        html += "<li class='breadcrumb-item active' style='font-weight: bold'>" + lastCrumb.name + "</li>";
        //console.log(crumbs);

        $.get("/query/children/" + siteId, (children) => {
            if (children.length > 0) {
                html += "<li class='breadcrumb-item'><select id='childList' onchange='selectBreadcrumbChild()'><option value='' style='font-style: italic'>Children...</option>";
                for (let i=0; i<children.length; i++) {
                    switch (children[i].level_type) {
                        case "ap": html += "<option value='/access_point/" + children[i].id + "'>" + children[i].name + "</option>"; break;
                        case "client": html += "<option value='/client/" + children[i].id + "'>" + children[i].name + "</option>"; break;
                        default: html += "<option value='/site/" + children[i].id + "'>" + children[i].name + "</option>"; break;
                    }
                }
                html += "</select></li>";
            }
            html += "</ol></nav>";
            $("#breadcrumbs").html(html);
        });
    });
}

function percentColor(pct) {
    if (pct < 50) {
        return "green";
    } else if (pct < 75) {
        return "orange";
    } else {
        return "red";
    }
}

function latencyColor(pct) {
    if (pct < 100) {
        return "green";
    } else if (pct < 150) {
        return "orange";
    } else {
        return "red";
    }
}

function peakBandwidth(siteId, div) {
    $.get("/query/node/" + siteId, (site) => {
        $.get("/query/peak_bandwidth/" + siteId, (data) => {
            let downPercent = (data.down / site.down_mbps) * 100;
            let upPercent = (data.up / site.up_mbps) * 100;
            let downColor = percentColor(downPercent);
            let upColor = percentColor(upPercent);
            let html = "<strong>Download</strong>: <span style='color: " + downColor + "'>" + data.down.toFixed(0) + " / " + site.down_mbps.toFixed(0) + " Mbps (" + downPercent.toFixed(0) + "%)</span><br />";
            html += "<strong>Upload</strong>: <span style='color: " + upColor + "'>" + data.up.toFixed(0) + " / " + site.up_mbps.toFixed(0) + " Mbps (" + upPercent.toFixed(0) + "%)</span>";
            $("#" + div).html(html);
        });
    });
}