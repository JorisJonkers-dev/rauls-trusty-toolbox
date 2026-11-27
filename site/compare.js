// SPDX-License-Identifier: GPL-3.0-only
// Wipe sliders: the transparent range input over each picture is the control (keyboard and touch come with it);
// the picture only follows its value. Without script both halves still show, split in the middle.
(function () {
  "use strict";
  document.querySelectorAll(".cmp").forEach(function (box) {
    var range = box.querySelector("input[type=range]");
    function set() { box.style.setProperty("--pos", range.value + "%"); range.setAttribute("aria-valuetext", range.value + " percent vanilla"); }
    range.addEventListener("input", set);
    set();
  });
})();
