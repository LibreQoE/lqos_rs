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