var colorPreference = 0;

function metaverse_color_ramp(n) {
    if (n <= 9) {
        return "#32b08c";
    } else if (n <= 20) {
        return "#ffb94a";
    } else if (n <=50) {
        return "#f95f53";
    } else if (n <=70) {
        return "#bf3d5e";
    } else {
        return "#dc4e58";
    }
}

function regular_color_ramp(n) {
    if (n <= 100) {
        return "#aaffaa";
    } else if (n <= 150) {
        return "goldenrod";
    } else {
        return "#ffaaaa";
    }
}

function color_ramp(n) {
    if (colorPreference == 0) {
        return regular_color_ramp(n);
    } else {
        return metaverse_color_ramp(n);
    }
}

function bindColorToggle() {
    $("#toggleColors").on('click', () => {
        if (colorPreference == 0) {
            colorPreference = 1;
            $("#toggleColors").text("(metaverse colors)");
        } else {
            colorPreference = 0;
            $("#toggleColors").text("(regular colors)");
        }
    });
}

function updateHostCounts() {
    $.get("/api/host_counts", (hc) => {
        $("#shapedCount").text(hc[0]);
        $("#unshapedCount").text(hc[1]);
        setTimeout(updateHostCounts, 5000);
    });
}

function colorReloadButton() {
    $("#btnReload").on('click', () => {
        alert("I promise to write this soon");
    });
    $.get("/api/reload_required", (req) => {
        if (req) {
            $("#btnReload").addClass('btn-warning');
            $("#btnReload").css('color', 'darkred');
        } else {
            $("#btnReload").addClass('btn-secondary');
        }
    })
}

function scaleNumber(n) {
    if (n > 1000000000000) {
        return (n/1000000000000).toFixed(2) + "T";
    } else if (n > 1000000000) {
        return (n/1000000000).toFixed(2) + "G";
    } else if (n > 1000000) {
        return (n/1000000).toFixed(2) + "M";
    } else if (n > 1000) {
        return (n/1000).toFixed(2) + "K";
    }
    return n;
}