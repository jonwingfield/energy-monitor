/* globals d3, moment */
function getParameterByName(name, url) {
    if (!url) { url = window.location.href; }
    name = name.replace(/[\[\]]/g, "\\$&");
    var regex = new RegExp("[?&]" + name + "(=([^&#]*)|&|#|$)"),
        results = regex.exec(url);
    if (!results) { return null; }
    if (!results[2]) { return ''; }
    return decodeURIComponent(results[2].replace(/\+/g, " "));
}

function linearRegression(x, y){
		var lr = {};
		var n = y.length;
		var sum_x = 0;
		var sum_y = 0;
		var sum_xy = 0;
		var sum_xx = 0;
		var sum_yy = 0;
		
		for (var i = 0; i < y.length; i++) {
			
			sum_x += x[i];
			sum_y += y[i];
			sum_xy += (x[i]*y[i]);
			sum_xx += (x[i]*x[i]);
			sum_yy += (y[i]*y[i]);
		} 
		
		lr.slope = (n * sum_xy - sum_x * sum_y) / (n*sum_xx - sum_x * sum_x);
		lr.intercept = (sum_y - lr.slope * sum_x)/n;
		lr.r2 = Math.pow((n*sum_xy - sum_x*sum_y)/Math.sqrt((n*sum_xx-sum_x*sum_x)*(n*sum_yy-sum_y*sum_y)),2);
		
		return lr;
}

function httpGet(url) {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    xhr.open("GET", url);
    xhr.onload = () => resolve(xhr.responseText);
    xhr.onerror = () => reject(xhr.statusText);
    xhr.send();
  });
}

var xTrackingLines = [];
var xTrackingTexts = [];

function errorLine(g, gOptions, options) {
  console.log(gOptions.yExtents);
  if (options.value <= gOptions.yExtents[0] || options.value > gOptions.yExtents[1]) {
    return;
  }

  const y = gOptions.y(options.value);
   
  g.append("line")
    .attr("x1", 0)
    .attr("y1", y)
    .attr("x2", gOptions.width)
    .attr("y2", y)
    .style('opacity', 1)
    .style("stroke", options.color);
}

// returns slope, intercept and r-square of the line
var history_graph = function(data, chartId, yLabel, yUnit, data2, y2Label, y2Unit,
  { decimalPlaces = 1, avgPoints = 60, extents } = {}) {
  var sum = function(a) { return a.reduce((a,b) => a + b, 0); };
  var average = function (array) { return sum(array) / array.length; };
  var movingAverage = function(a, n) {
    return a.map((d,i) => {
      var startIndex = Math.max(i-n, 0);
      return {
        timestamp: d.timestamp,
        value: average(a.slice(startIndex, i + 1).map(d => d.value))
      };
    });
  };
  const chart = document.querySelector(chartId);

  chart.innerHTML = '';
  chart.setAttribute('width', document.documentElement.clientWidth - 20);

  var svg = d3.select(chartId),
    margin = { top: 20, right: 40, bottom: 30, left: 50 },
    width = +svg.attr("width") - margin.left - margin.right,
    height = +svg.attr("height") - margin.top - margin.bottom,
    g = svg.append("g").attr("transform", "translate(" + margin.left + "," + margin.top + ")");

  var x = d3.scaleTime()
    .rangeRound([0, width]);

  var y = d3.scaleLinear()
    .rangeRound([height, 0]);

  var line = d3.line()
    .x(d => x(d.timestamp))
    .y(d => y(d.value));

  var xDomain = d3.extent(data, d => d.timestamp);
  x.domain(xDomain);
  let yExtents = d3.extent(data, d => d.value);
  if (extents) {
    yExtents = extents(yExtents);
  }
  y.domain(yExtents);

  g.append("g")
      .attr("transform", `translate(0,${height})`)
      .attr("class", "axis")
      .call(d3.axisBottom(x).ticks(Math.ceil(document.documentElement.clientWidth / 100.0)));

  g.append("g")
      .attr("class", "axis")
      .call(d3.axisLeft(y))
    .append("text")
      .attr("transform", "rotate(-90)")
      .attr("y", 6)
      .attr("dy", "0.71em")
      .attr("text-anchor", "end")
      .text(`${yLabel} (${yUnit})`);

  var path = g.append("path")
    .attr("class", "line")
    .attr("fill", "none")
    .attr("stroke", "#00d700")
    .attr("stroke-linejoin", "round")
    .attr("stroke-linecap", "round")
    .attr("stroke-width", 1.0)
    .attr("d", line(data));

  if (avgPoints > 0) {
    g.append("path")
      .attr("class", "line")
      .attr("fill", "none")
      .attr("stroke", "orange")
      .attr("stroke-linejoin", "round")
      .attr("stroke-linecap", "round")
      .attr("stroke-width", 1.0)
      .attr("d", line(movingAverage(data, avgPoints)));
  }

  // var regressionLine = linearRegression(data.map(d => x(d.timestamp)), data.map(d => y(d.value)));
  // var max = d3.max(data, d => x(d.timestamp));

  // g.append("line")
  //   .attr("x1", x(data[0].timestamp))
  //   .attr("y1", regressionLine.intercept)
  //   .attr("x2", max)
  //   .attr("y2", (max * regressionLine.slope) + regressionLine.intercept)
  //   .style("stroke", "black");

  if (data2) {
    var y2 = d3.scaleLinear()
      .rangeRound([height, 0]);
    var extent = d3.extent(data2, d => d.value);
    var buffer = (extent[1] - extent[0]) * 0.2;
    y2.domain([extent[0], extent[1] + buffer]);

    var line2 = d3.line()
      .x(d => x(d.timestamp))
      .y(d => y2(d.value));

    g.append("g")
      .attr('transform', `translate(${width}, 0)`)
      .attr("class", "axis")
      .call(d3.axisRight(y))
      .append("text")
      .attr("fill", "#000")
      .attr("transform", "rotate(-90)")
      .attr("y", -12)
      .attr("dy", "0.71em")
      .attr("text-anchor", "end")
      .text(`${yLabel} (${yUnit})`);

    g.append("path")
      // .attr('transform', "translate(0, " + height + ")")
      .attr("class", "line")
      .attr("fill", "none")
      .attr("stroke", "orange")
      .attr("stroke-linejoin", "round")
      .attr("stroke-linecap", "round")
      .attr("stroke-width", 1.0)
      .attr("d", line2(data2));

    // g.append("path")
    //   .attr("class", "line")
    //   .attr("fill", "none")
    //   .attr("stroke", "purple")
    //   .attr("stroke-linejoin", "round")
    //   .attr("stroke-linecap", "round")
    //   .attr("stroke-width", 2.0)
    //   .style("opacity", 0.6)
    //   .attr("d", line2(movingAverage(data2, 10)));
  }

  g.append("text")
    .attr("class", "main-value")
    .attr("x", 30)
    .attr("y", 20)
    .text(data[data.length - 1].value.toFixed(1) + "" + yUnit + " " + (data2 ? Math.round(data2[data2.length-1].value) + "" + y2Unit : ""));

  var trackingLine = g.append("line")
    .attr("class", "tracking")
    .attr("x1", 0)
    .attr("y1", 0)
    .attr("x2", width)
    .attr("y2", 0)
    .style('opacity', 0);

  xTrackingLines.push(g.append("line")
    .attr("class", "tracking")
    .attr("x1", 0)
    .attr("y1", 0)
    .attr("x2", 0)
    .attr("y2", height)
    .style('opacity', 0));

  xTrackingTexts.push(g.append("text")
        .attr("class", "tracking")
        .attr("y", 0)
        .attr("x", 0)
        .attr("dy", "0.71em")
        .attr("text-anchor", "start")
        .style("opacity", 0)
        .text(""));

  var trackingText = g.append("text")
        .attr("class", "tracking")
        .attr("y", 0)
        .attr("x", width - 10)
        .attr("dy", "0.71em")
        .attr("text-anchor", "end")
        .style("opacity", 0)
        .text("");

  svg.on('mousemove', function() {
    var pos = d3.mouse(this),
      posx = pos[0] - margin.left,
      posy = pos[1] - margin.top;
     
    if (posy < 0 ||
      posy > height) {
      trackingLine.style('opacity', 0);
      trackingText.style('opacity', 0);
    } else {
      trackingLine.style('opacity', 1);
      trackingText.style('opacity', 1);
    }

    if (posx < 0 || posx > width) {
      xTrackingLines.forEach(xTrackingLine => xTrackingLine.style('opacity', 0));
      xTrackingTexts.forEach(xTrackingText => xTrackingText.style('opacity', 0));
    } else {
      xTrackingLines.forEach(xTrackingLine => xTrackingLine.style('opacity', 1));
      xTrackingTexts.forEach(xTrackingText => xTrackingText.style('opacity', 1));
    }

    var xValue = moment(x.invert(posx)).format("h:mm a");
    var value = (Math.round(y.invert(posy) * Math.pow(10, decimalPlaces)) / Math.pow(10, decimalPlaces)).toFixed(decimalPlaces);

    trackingLine
      // .style('opacity', 1)
      .attr('y1', posy)
      .attr('y2', posy);
    trackingText
      .attr('y', posy - 15)
      .text(value);

    xTrackingLines.forEach(xTrackingLine => 
      xTrackingLine
        .attr('x1', posx)
        .attr('x2', posx));
    xTrackingTexts.forEach(xTrackingText => 
      xTrackingText
        .attr('x', posx + 3)
        .text(xValue));
  });

  return [g, { x, y, width, height, yExtents }];
};

var mapValues = function(values, index, valueCallback) {
  var parseTime = d3.utcParse("%Y-%m-%dT%H:%M:%S.%LZ");
  valueCallback = valueCallback || (v => v);
  var rows = d3.csvParseRows(values, function(d/*, i*/) {
    var val = d[index];
    var time = parseTime(d[0]);
    if (!val || !time) { return null; }
    return {
      timestamp: time,
      value: valueCallback(parseFloat(d[index]))
    };
  });
  var min = moment().startOf('day');
  rows.splice(0, rows.findIndex(r => r.timestamp > min));
  return rows;
};


var update = function() {
  xTrackingLines = [];
  xTrackingTexts = [];

  const file = (getParameterByName('date') || 'today') + '.csv';
  httpGet(file).then(function(csv) { 
    let min;
    const adjustBaseline = val => {
      if (min === undefined) {
        min = val;
      }
      return val - min;
    };
    history_graph(mapValues(csv, 3), '#energy_graph', "Power", "W", mapValues(csv, 5, adjustBaseline), "Energy", "Wh",
      { decimalPlaces: 0, avgPoints: 0 }); 
    history_graph(mapValues(csv, 7), '#house_energy_graph', "Power", "W", mapValues(csv, 9, adjustBaseline), "Energy", "Wh",
      { decimalPlaces: 0, avgPoints: 0 }); 
    const voltage_graph = history_graph(mapValues(csv, 1), '#voltage_graph', "Battery", "V", null, null, null,
      { decimalPlaces: 2, avgPoints: 0, extents: ([min, max]) => [min - 0.2, max + 0.1] }); 
    errorLine.apply(window, voltage_graph.concat({ value: 3.65*4, color: 'orange' }));
    errorLine.apply(window, voltage_graph.concat({ value: 3.5*4, color: 'red' }));
    errorLine.apply(window, voltage_graph.concat({ value: 4.1*4, color: 'orange' }));
    errorLine.apply(window, voltage_graph.concat({ value: 4.15*4, color: 'red' }));
  });
};

window.setInterval(update, 30000);
update();

// function limit_data(h) {
//   history.replaceState({}, "", "history.html?hours=" + h);
//     point_limit = 60*2*h;
//     hours = h;
//     update();
// }

// $(function() {
//   $('#two_weeks').click(function(e) {
//     e.preventDefault();
//     limit_data(24*7*2);
//   });
//   $('#one_week').click(function(e) {
//     e.preventDefault();
//     limit_data(24*7);
//   });

//   $('#two_days').click(function(e) {
//     e.preventDefault();
//     limit_data(48);
//   });

//   $('#one_day').click(function(e) {
//     e.preventDefault();
//     limit_data(24);
//   });

//   $('#half_day').click(function(e) {
//     e.preventDefault();
//     limit_data(12);
//   });
//   $('#quarter_day').click(function(e) {
//     e.preventDefault();
//     limit_data(6);
//   });
// });
