// ═══════════════════════════════════════════════════════
// Hadal Zone Panel — 式舆防卫战
// ═══════════════════════════════════════════════════════

async function renderHadalZone() {
  var hz = await API.getHadalZone(state.uid);
  if (!hz) { $('#main').innerHTML = '<div class="empty-state"><p>未找到式舆防卫战数据</p></div>'; return; }

  var html = '<div class="page-header"><h2>式舆防卫战</h2><span class="subtitle">修改 Zone ID 以切换期号，服务器热加载即时生效</span></div>';

  html += '<div class="section-title">入口配置</div>';
  html += '<div class="entrance-grid">';

  var entranceNames = { 1: '危局强袭站', 2: '式舆防卫战·稳定', 3: '式舆防卫战·剧变', 9: '式舆防卫战·特殊' };
  var entranceIcons = { 1: '◆', 2: '◇', 3: '◆', 9: '◇' };

  for (var i = 0; i < hz.entrances.length; i++) {
    var e = hz.entrances[i];
    var name = entranceNames[e.id] || '入口 ' + e.id;
    html += '<div class="entrance-card">' +
      '<div class="entrance-card__icon">' + (entranceIcons[e.id] || '◆') + '</div>' +
      '<div class="entrance-card__info">' +
        '<div class="entrance-card__name">' + name + '</div>' +
        '<div class="entrance-card__type">' + (e.id === 2 || e.id === 3 ? '常驻' : '限时') + ' · ID: ' + e.id + '</div>' +
      '</div>' +
      '<div>' +
        '<label class="form-label text-center">Zone ID</label>' +
        '<input type="number" class="hz-zone" data-id="' + e.id + '" value="' + e.zone_id + '">' +
      '</div>' +
    '</div>';
  }
  html += '</div>';

  html += '<div class="btn-group"><button class="btn btn-primary" data-action="save-hadal-zone">保存更改</button></div>';

  if (hz.saved_rooms && hz.saved_rooms.length > 0) {
    html += '<div class="section-title mt-4">已保存的房间</div>';
    html += '<div class="panel-box"><table class="data-table"><thead><tr><th>Zone</th><th>Layer</th><th>Avatars</th><th>Buddy</th></tr></thead><tbody>';
    for (var ri = 0; ri < hz.saved_rooms.length; ri++) {
      var room = hz.saved_rooms[ri];
      html += '<tr><td>' + room.zone_id + '</td><td>' + room.layer_index + '</td><td>' + ((room.avatar_id_list || []).join(', ') || '—') + '</td><td>' + (room.buddy_id || 0) + '</td></tr>';
    }
    html += '</tbody></table></div>';
  }

  $('#main').innerHTML = html;
}

async function saveHadalZone() {
  await safeSave('式舆防卫战配置', async function() {
    var entrances = [];
    $$('.hz-zone').forEach(function(inp) {
      entrances.push({ id: parseInt(inp.dataset.id), zone_id: parseInt(inp.value) || 0 });
    });
    var hz = await API.getHadalZone(state.uid);
    await API.updateHadalZone(state.uid, { entrances: entrances, saved_rooms: hz.saved_rooms });
  });
}
