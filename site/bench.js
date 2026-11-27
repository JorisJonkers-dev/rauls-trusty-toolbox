// SPDX-License-Identifier: GPL-3.0-only
// The benchmark pages draw complete without this file: every mark has its value in the SVG and a <title>. This adds one
// tooltip for hover, touch and keyboard focus, showing the value, the series and the run.
"use strict";
(function () {
  var tip = document.createElement("div");
  tip.className = "bx-tip";
  tip.setAttribute("role", "status");
  tip.setAttribute("aria-live", "polite");
  var bv = document.createElement("b"), bt = document.createElement("span"), bm = document.createElement("i");
  tip.appendChild(bv); tip.appendChild(bt); tip.appendChild(bm);
  document.body.appendChild(tip);
  var marks = document.querySelectorAll("[data-tip]");
  /* the native tooltip would show next to ours: its text stays as the mark's accessible name */
  for (var i = 0; i < marks.length; i++) {
    var t = marks[i].querySelector(":scope > title");
    if (t) t.remove();
  }
  var current = null;
  function show(el) {
    current = el;
    bv.textContent = el.getAttribute("data-tip-v") || "";
    bt.textContent = el.getAttribute("data-tip") || "";
    bm.textContent = el.getAttribute("data-tip-m") || "";
    bm.style.display = bm.textContent ? "" : "none";
    var mark = el.querySelector(".bx-m, .bx-dot, .bx-slot") || el;
    var r = mark.getBoundingClientRect();
    tip.classList.add("on");
    var w = tip.offsetWidth, h = tip.offsetHeight;
    var x = Math.min(Math.max(8, r.left + r.width / 2 - w / 2), window.innerWidth - w - 8);
    var y = r.top - h - 10;
    if (y < 8) y = r.bottom + 10;
    tip.style.left = x + "px";
    tip.style.top = y + "px";
  }
  function hide(el) { if (!el || el === current) { tip.classList.remove("on"); current = null; } }
  function find(e) { return e.target && e.target.closest ? e.target.closest("[data-tip]") : null; }
  document.addEventListener("pointerover", function (e) { var el = find(e); if (el) show(el); });
  document.addEventListener("pointerout", function (e) { var el = find(e); if (el && !el.contains(e.relatedTarget)) hide(el); });
  document.addEventListener("focusin", function (e) { var el = find(e); if (el) show(el); else hide(); });
  document.addEventListener("focusout", function (e) { var el = find(e); if (el) hide(el); });
  document.addEventListener("keydown", function (e) { if (e.key === "Escape") hide(); });
  /* the tooltip stays with its mark when the page or a figure scrolls */
  window.addEventListener("scroll", function () { if (current) show(current); }, { passive: true, capture: true });
})();
