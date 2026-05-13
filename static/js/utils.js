// ═══════════════════════════════════════════════════════
// DOM Utilities & Rendering Helpers
// ═══════════════════════════════════════════════════════

// Query helpers
var $ = function(s) { return document.querySelector(s); };
var $$ = function(s) { return document.querySelectorAll(s); };

// HTML escape
function escHtml(s) {
  var d = document.createElement('div');
  d.textContent = s;
  return d.innerHTML;
}

// ═══════════════════════════════════════════════════════
// Toast notification
// ═══════════════════════════════════════════════════════
function toast(msg, type) {
  if (!type) type = 'success';
  var container = $('#toast-container');
  var el = document.createElement('div');
  el.className = 'toast toast--' + type;
  el.setAttribute('role', 'alert');
  el.innerHTML = '<span class="toast-icon">' + (type === 'success' ? '✓' : (type === 'error' ? '✗' : 'ℹ')) + '</span><span class="toast__msg">' + escHtml(msg) + '</span><button class="toast__close" data-action="close-toast" aria-label="关闭">&times;</button>';
  container.appendChild(el);
  var timer = setTimeout(function() { removeToast(el); }, 3000);
  el._timer = timer;
}

function removeToast(el) {
  clearTimeout(el._timer);
  el.style.opacity = '0';
  el.style.transform = 'translateY(-20px)';
  el.style.transition = 'all 0.25s ease';
  setTimeout(function() { if (el.parentNode) el.remove(); }, 250);
}

// ═══════════════════════════════════════════════════════
// Custom Confirm Dialog
// ═══════════════════════════════════════════════════════
function showConfirm(msg, onConfirm) {
  var overlay = document.createElement('div');
  overlay.className = 'modal-overlay confirm-overlay';
  overlay._onConfirm = onConfirm;
  overlay.innerHTML = '<div class="confirm-dialog">' +
    '<p class="confirm-msg">' + escHtml(msg) + '</p>' +
    '<div class="confirm-actions">' +
    '<button class="btn btn-ghost" data-action="confirm-cancel">取消</button>' +
    '<button class="btn btn-danger" data-action="confirm-ok">确认</button>' +
    '</div></div>';
  document.body.appendChild(overlay);
  overlay.addEventListener('click', function(e) { if (e.target === overlay) overlay.remove(); });
}

// ═══════════════════════════════════════════════════════
// Safe Save Wrapper
// ═══════════════════════════════════════════════════════
async function safeSave(label, fn) {
  try {
    // Disable save buttons during save
    var saveBtns = document.querySelectorAll('[data-action^="save-"]');
    saveBtns.forEach(function(b) { b.disabled = true; b.textContent = '保存中...'; });
    await fn();
    _dirty = false;
    toast(label + ' 已保存');
    return true;
  }
  catch(e) { toast(label + ' 失败: ' + escHtml(e.message || String(e)), 'error'); return false; }
  finally {
    var saveBtns = document.querySelectorAll('[data-action^="save-"]');
    saveBtns.forEach(function(b) { b.disabled = false; b.textContent = '保存更改'; });
  }
}

// ═══════════════════════════════════════════════════════
// Search handler factory (dedup)
// ═══════════════════════════════════════════════════════
function createSearchHandler(stateKey, viewKey, renderFn, selectedKey) {
  var timer = 0;
  return function(val) {
    state[stateKey] = val;
    state[viewKey] = 'gallery';
    if (selectedKey) state[selectedKey] = null;
    clearTimeout(timer);
    var activeEl = document.activeElement;
    var activeId = activeEl ? activeEl.id : null;
    timer = setTimeout(function() {
      renderFn();
      // Restore focus to search input after re-render
      if (activeId) {
        var restored = document.getElementById(activeId);
        if (restored) { restored.focus(); restored.setSelectionRange(restored.value.length, restored.value.length); }
      }
    }, 150);
  };
}

// ═══════════════════════════════════════════════════════
// Star / Sub-dot / Rank rendering
// ═══════════════════════════════════════════════════════
function renderStars(count, max) {
  var s = '<span class="star-rating">';
  for (var i = 0; i < max; i++) {
    s += '<span class="star ' + (i < count ? 'on' : 'off') + '">★</span>';
  }
  s += '</span>';
  return s;
}

function renderSubDots(count, max) {
  var s = '<span class="sub-dot-group">';
  for (var i = 0; i < max; i++) {
    s += '<span class="sub-dot ' + (i < count ? 'filled' : '') + '"></span>';
  }
  s += '</span>';
  return s;
}

function renderRankDots(rank) {
  var s = '<span class="rank-dots">';
  for (var i = 0; i < 6; i++) {
    s += '<span class="rank-dot ' + (i < rank ? 'filled' : '') + '"></span>';
  }
  s += '</span>';
  return s;
}

// ═══════════════════════════════════════════════════════
// Stepper Input HTML Generators
// ═══════════════════════════════════════════════════════
function stepperInput(id, value, min, max) {
  return '<div class="input-stepper">' +
    '<button type="button" data-action="step" data-selector="#' + id + '" data-delta="-1" data-min="' + min + '" data-max="' + max + '" aria-label="减少">−</button>' +
    '<input type="number" id="' + id + '" value="' + value + '" min="' + min + '" max="' + max + '" data-action="clamp-input" data-min="' + min + '" data-max="' + max + '">' +
    '<button type="button" data-action="step" data-selector="#' + id + '" data-delta="1" data-min="' + min + '" data-max="' + max + '" aria-label="增加">+</button>' +
    '</div>';
}

// ═══════════════════════════════════════════════════════
// Stepper Logic Functions
// ═══════════════════════════════════════════════════════

// Simple step by element ID (legacy, used directly in some places)
function stepInput(id, delta, min, max) {
  var el = document.getElementById(id);
  if (!el) return;
  var v = (parseInt(el.value) || 0) + delta;
  el.value = Math.max(min, Math.min(max, v));
}

// Clamp input value on change
function clampInput(id, min, max) {
  var el = document.getElementById(id);
  if (!el) return;
  var v = parseInt(el.value) || 0;
  if (v < min) el.value = min;
  if (v > max) el.value = max;
}

// Step value by ID (used by equip editor) — dispatches input event for live updates
function stepValue(id, delta) {
  var el = document.getElementById(id);
  if (!el) return;
  var v = parseInt(el.value) || 0;
  v = Math.max(parseInt(el.min) || 0, Math.min(parseInt(el.max) || 999, v + delta));
  el.value = v;
  el.dispatchEvent(new Event('input', {bubbles: true}));
}

// ═══════════════════════════════════════════════════════
// Unified Stepper by CSS Selector (Task 4.4)
// Replaces stepEqSubBase, stepEqSubAdd, stepNewEqSubBase, stepNewEqSubAdd
// Usage: stepBySelector('.eq-sub-add[data-idx="0"]', 1, 0, 4)
// ═══════════════════════════════════════════════════════
function stepBySelector(selector, delta, min, max) {
  var el = document.querySelector(selector);
  if (!el) return;
  var v = parseInt(el.value) || 0;
  v = Math.max(min, Math.min(max, v + delta));
  el.value = v;
  el.dispatchEvent(new Event('input', {bubbles: true}));
}

// Stepper keyboard support: up/down arrows on number inputs
document.addEventListener('keydown', function(e) {
  if (e.target.tagName !== 'INPUT' || e.target.type !== 'number') return;
  if (e.key === 'ArrowUp') {
    e.preventDefault();
    stepBySelector('#' + e.target.id, 1, parseInt(e.target.min) || 0, parseInt(e.target.max) || 999);
  } else if (e.key === 'ArrowDown') {
    e.preventDefault();
    stepBySelector('#' + e.target.id, -1, parseInt(e.target.min) || 0, parseInt(e.target.max) || 999);
  }
});

// ═══════════════════════════════════════════════════════
// Window exports (minimal — only for inline onclick compatibility)
// ═══════════════════════════════════════════════════════
