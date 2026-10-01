// GitHub Desktop - Pure Rust + Tauri Frontend Engine
const invoke = async (cmd, args = {}) => {
  if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === 'function') {
    return window.__TAURI__.core.invoke(cmd, args);
  }
  if (window.__TAURI__ && typeof window.__TAURI__.invoke === 'function') {
    return window.__TAURI__.invoke(cmd, args);
  }
  if (window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === 'function') {
    return window.__TAURI_INTERNALS__.invoke(cmd, args);
  }
  console.warn(`Mock invocation for ${cmd}:`, args);
  return null;
};

// Application State
const state = {
  activeTab: 'changes',
  selectedFile: null,
  selectedCommit: null,
  status: {
    branch: 'main',
    upstream: null,
    ahead: 0,
    behind: 0,
    files: [],
  },
  commits: [],
  branches: [],
  repositories: [],
  pullRequests: [],
  stashes: [],
  selectedMergeBranch: null,
  commitFiles: [],
  selectedCommitFile: null,
  isSyncing: false,
};

// DOM References
const el = {
  repoBtn: document.getElementById('repo-btn'),
  repoName: document.getElementById('repo-name'),
  branchName: document.getElementById('branch-name'),
  aheadBehind: document.getElementById('ahead-behind'),
  syncTitle: document.getElementById('sync-title'),
  syncSublabel: document.getElementById('sync-sublabel'),
  syncSpinner: document.getElementById('sync-spinner'),
  syncBtn: document.getElementById('sync-btn'),
  branchBtn: document.getElementById('branch-btn'),
  prsBtn: document.getElementById('prs-btn'),
  terminalBtn: document.getElementById('terminal-btn'),

  repoModal: document.getElementById('repo-modal'),
  repoModalClose: document.getElementById('repo-modal-close'),
  repoSearchInput: document.getElementById('repo-search-input'),
  modalRepoList: document.getElementById('modal-repo-list'),
  repoModalAddBtn: document.getElementById('repo-modal-add-btn'),
  repoModalNewBtn: document.getElementById('repo-modal-new-btn'),
  repoModalCloneBtn: document.getElementById('repo-modal-clone-btn'),

  tabChanges: document.getElementById('tab-changes'),
  tabHistory: document.getElementById('tab-history'),
  viewChanges: document.getElementById('view-changes'),
  viewHistory: document.getElementById('view-history'),
  changesCountBadge: document.getElementById('changes-count'),
  changesSummaryCount: document.getElementById('changes-summary-count'),
  selectAllCheckbox: document.getElementById('select-all-checkbox'),
  fileList: document.getElementById('file-list'),

  stashBanner: document.getElementById('stash-banner'),
  stashBannerText: document.getElementById('stash-banner-text'),
  stashRestoreBtn: document.getElementById('stash-restore-btn'),
  stashDiscardBtn: document.getElementById('stash-discard-btn'),

  commitSummary: document.getElementById('commit-summary'),
  commitDescription: document.getElementById('commit-description'),
  commitBtn: document.getElementById('commit-btn'),
  commitBranchLabel: document.getElementById('commit-branch-label'),
  undoBtn: document.getElementById('undo-btn'),

  historyFilter: document.getElementById('history-filter'),
  commitList: document.getElementById('commit-list'),

  commitHeader: document.getElementById('commit-header'),
  commitHeaderTitle: document.getElementById('commit-header-title'),
  commitHeaderAuthor: document.getElementById('commit-header-author'),
  commitHeaderDate: document.getElementById('commit-header-date'),
  commitHeaderSha: document.getElementById('commit-header-sha'),
  commitHeaderDesc: document.getElementById('commit-header-desc'),
  copyShaBtn: document.getElementById('copy-sha-btn'),
  historyUndoBtn: document.getElementById('history-undo-btn'),
  historyRevertBtn: document.getElementById('history-revert-btn'),

  commitFilesBar: document.getElementById('commit-files-bar'),
  commitFilesSummary: document.getElementById('commit-files-summary'),
  commitFilesList: document.getElementById('commit-files-list'),

  diffHeader: document.getElementById('diff-header'),
  diffFilename: document.getElementById('diff-filename'),
  statAdd: document.getElementById('stat-add'),
  statDel: document.getElementById('stat-del'),
  diffContent: document.getElementById('diff-content'),
  openEditorBtn: document.getElementById('open-editor-btn'),
  revealFinderBtn: document.getElementById('reveal-finder-btn'),
  discardFileBtn: document.getElementById('discard-file-btn'),

  branchModal: document.getElementById('branch-modal'),
  branchModalClose: document.getElementById('branch-modal-close'),
  branchSearchInput: document.getElementById('branch-search-input'),
  modalBranchList: document.getElementById('modal-branch-list'),
  newBranchName: document.getElementById('new-branch-name'),
  createBranchBtn: document.getElementById('create-branch-btn'),

  mergeModal: document.getElementById('merge-modal'),
  mergeModalClose: document.getElementById('merge-modal-close'),
  mergeTargetBranch: document.getElementById('merge-target-branch'),
  mergeTargetBranchText: document.getElementById('merge-target-branch-text'),
  mergeSearchInput: document.getElementById('merge-search-input'),
  modalMergeBranchList: document.getElementById('modal-merge-branch-list'),
  mergeCancelBtn: document.getElementById('merge-cancel-btn'),
  mergeConfirmBtn: document.getElementById('merge-confirm-btn'),

  prModal: document.getElementById('pr-modal'),
  prModalClose: document.getElementById('pr-modal-close'),
  prList: document.getElementById('pr-list'),

  newRepoModal: document.getElementById('new-repo-modal'),
  newRepoModalClose: document.getElementById('new-repo-modal-close'),
  newRepoNameInput: document.getElementById('new-repo-name-input'),
  newRepoPathInput: document.getElementById('new-repo-path-input'),
  newRepoReadmeCheckbox: document.getElementById('new-repo-readme-checkbox'),
  newRepoCancelBtn: document.getElementById('new-repo-cancel-btn'),
  newRepoCreateBtn: document.getElementById('new-repo-create-btn'),

  addRepoModal: document.getElementById('add-repo-modal'),
  addRepoModalClose: document.getElementById('add-repo-modal-close'),
  addRepoPathInput: document.getElementById('add-repo-path-input'),
  addRepoCancelBtn: document.getElementById('add-repo-cancel-btn'),
  addRepoConfirmBtn: document.getElementById('add-repo-confirm-btn'),

  cloneRepoModal: document.getElementById('clone-repo-modal'),
  cloneRepoModalClose: document.getElementById('clone-repo-modal-close'),
  cloneRepoUrlInput: document.getElementById('clone-repo-url-input'),
  cloneRepoDestInput: document.getElementById('clone-repo-dest-input'),
  cloneRepoCancelBtn: document.getElementById('clone-repo-cancel-btn'),
  cloneRepoConfirmBtn: document.getElementById('clone-repo-confirm-btn'),

  settingsModal: document.getElementById('settings-modal'),
  settingsModalClose: document.getElementById('settings-modal-close'),
  settingsUserName: document.getElementById('settings-user-name'),
  settingsUserEmail: document.getElementById('settings-user-email'),
  settingsEditorSelect: document.getElementById('settings-editor-select'),
  settingsCancelBtn: document.getElementById('settings-cancel-btn'),
  settingsSaveBtn: document.getElementById('settings-save-btn'),

  aboutModal: document.getElementById('about-modal'),
  aboutModalCloseBtn: document.getElementById('about-modal-close-btn'),

  toast: document.getElementById('toast'),
};

// Initialization
async function init() {
  try {
    setupEventListeners();
  } catch (err) {
    console.error('Failed to set up event listeners:', err);
  }
  try {
    await refreshRepoInfo();
  } catch (err) {
    console.error('Failed to refresh repo info:', err);
  }
  try {
    await refreshStatus();
  } catch (err) {
    console.error('Failed to refresh status:', err);
  }
  try {
    await loadCommits();
  } catch (err) {
    console.error('Failed to load commits:', err);
  }
  try {
    await refreshStash();
  } catch (err) {
    console.error('Failed to refresh stash:', err);
  }
}

async function refreshStash() {
  try {
    const stashes = await invoke('get_stashes');
    state.stashes = stashes || [];
    const currBranch = state.status?.branch || 'main';
    const branchStash = state.stashes.find(s => s.branch === currBranch || s.branch === 'HEAD');
    if (branchStash) {
      el.stashBannerText.textContent = `Stashed changes on ${branchStash.branch} (${branchStash.message})`;
      el.stashBanner.style.display = 'flex';
      el.stashBanner.dataset.index = branchStash.index;
    } else if (state.stashes.length > 0) {
      el.stashBannerText.textContent = `1 stash entry available (${state.stashes[0].message})`;
      el.stashBanner.style.display = 'flex';
      el.stashBanner.dataset.index = state.stashes[0].index;
    } else {
      el.stashBanner.style.display = 'none';
    }
  } catch (err) {
    console.error('Failed to get stashes:', err);
  }
}

function showToast(message, isError = false) {
  el.toast.textContent = message;
  el.toast.className = `toast show ${isError ? 'error' : ''}`;
  setTimeout(() => {
    el.toast.className = 'toast';
  }, 3200);
}

// Data Loaders
async function refreshRepoInfo() {
  try {
    const summary = await invoke('get_repo_summary');
    if (summary) {
      el.repoName.textContent = summary.name;
      el.branchName.textContent = summary.branch;
      el.commitBranchLabel.textContent = summary.branch;

      if (summary.ahead > 0 || summary.behind > 0) {
        el.aheadBehind.textContent = `↑${summary.ahead} ↓${summary.behind}`;
        el.aheadBehind.style.display = 'inline-block';
      } else {
        el.aheadBehind.style.display = 'none';
      }

      if (summary.behind > 0) {
        el.syncTitle.textContent = `Pull ${summary.behind} commits`;
        el.syncSublabel.textContent = 'Pull origin';
      } else if (summary.ahead > 0) {
        el.syncTitle.textContent = `Push ${summary.ahead} commits`;
        el.syncSublabel.textContent = 'Push origin';
      } else {
        el.syncTitle.textContent = 'Fetch origin';
        el.syncSublabel.textContent = 'Fetch origin';
      }
    }
  } catch (err) {
    console.error('Failed to get repo summary:', err);
  }
}

async function refreshStatus() {
  try {
    const status = await invoke('get_status');
    state.status = status;

    el.changesCountBadge.textContent = status.files.length;
    el.changesSummaryCount.textContent = `${status.files.length} changed file${status.files.length === 1 ? '' : 's'}`;

    const anyUnstaged = status.files.some(f => f.unstaged_status !== 'Unmodified' || f.staged_status === 'Untracked');
    el.selectAllCheckbox.checked = !anyUnstaged && status.files.length > 0;

    renderFileList(status.files);

    if (state.selectedFile && status.files.some(f => f.path === state.selectedFile)) {
      await loadDiff(state.selectedFile);
    } else if (status.files.length > 0) {
      selectFile(status.files[0].path);
    } else {
      state.selectedFile = null;
      renderEmptyState();
    }
  } catch (err) {
    console.error('Failed to get status:', err);
  }
}

async function loadCommits() {
  try {
    const commits = await invoke('get_commits', { limit: 50 });
    state.commits = commits || [];
    const filter = el.historyFilter ? el.historyFilter.value.trim() : '';
    renderCommitList(state.commits, filter);
    if (state.commits.length > 0) {
      const selected = state.commits.find(c => c.sha === state.selectedCommit);
      if (selected) {
        selectCommit(selected, selected.sha === state.commits[0].sha);
      } else if (state.activeTab === 'history') {
        selectCommit(state.commits[0], true);
      }
    }
  } catch (err) {
    console.error('Failed to load commits:', err);
  }
}

async function loadDiff(filePath, staged = false) {
  try {
    const diffs = await invoke('get_diff', { filePath, staged });
    renderDiff(diffs, filePath);
  } catch (err) {
    console.error('Failed to load diff:', err);
    el.diffContent.innerHTML = `<div class="empty-state"><p>Error loading diff: ${err}</p></div>`;
  }
}

// Rendering
function renderFileList(files) {
  el.fileList.innerHTML = '';

  if (files.length === 0) {
    el.fileList.innerHTML = `<div style="padding: 24px; text-align: center; color: var(--text-muted); font-size: 12px;">Working directory clean</div>`;
    return;
  }

  files.forEach(file => {
    const isStaged = file.staged_status !== 'Unmodified' && file.staged_status !== 'Untracked';
    const isSelected = file.path === state.selectedFile;

    const div = document.createElement('div');
    div.className = `file-item ${isSelected ? 'selected' : ''}`;

    let statusChar = 'M';
    let statusClass = 'status-M';
    if (file.staged_status === 'Added' || file.unstaged_status === 'Added') {
      statusChar = 'A';
      statusClass = 'status-A';
    } else if (file.staged_status === 'Deleted' || file.unstaged_status === 'Deleted') {
      statusChar = 'D';
      statusClass = 'status-D';
    } else if (file.staged_status === 'Untracked' || file.unstaged_status === 'Untracked') {
      statusChar = '?';
      statusClass = 'status-Q';
    }

    const lastSlash = file.path.lastIndexOf('/');
    const basename = lastSlash >= 0 ? file.path.substring(lastSlash + 1) : file.path;
    const dirname = lastSlash >= 0 ? file.path.substring(0, lastSlash) : '';

    div.innerHTML = `
      <label class="checkbox-container" onclick="event.stopPropagation()">
        <input type="checkbox" class="file-checkbox" data-path="${file.path}" ${isStaged ? 'checked' : ''}>
        <span class="checkmark"></span>
      </label>
      <span class="status-badge ${statusClass}">${statusChar}</span>
      <div class="file-name-container">
        <span class="file-basename">${basename}</span>
        ${dirname ? `<span class="file-dirname">${dirname}</span>` : ''}
      </div>
    `;

    div.addEventListener('click', () => {
      selectFile(file.path);
    });

    const checkbox = div.querySelector('.file-checkbox');
    checkbox.addEventListener('change', async (e) => {
      const shouldStage = e.target.checked;
      if (shouldStage) {
        await invoke('stage_file', { path: file.path });
      } else {
        await invoke('unstage_file', { path: file.path });
      }
      await refreshStatus();
    });

    el.fileList.appendChild(div);
  });
}

function selectFile(filePath) {
  state.selectedFile = filePath;
  const items = el.fileList.querySelectorAll('.file-item');
  items.forEach(it => {
    const cb = it.querySelector('.file-checkbox');
    if (cb && cb.getAttribute('data-path') === filePath) {
      it.classList.add('selected');
    } else {
      it.classList.remove('selected');
    }
  });

  loadDiff(filePath);
}

function buildHunkPatch(filePath, hunk) {
  let patch = `--- a/${filePath}\n+++ b/${filePath}\n`;
  patch += hunk.header + '\n';
  hunk.lines.forEach(line => {
    let prefix = ' ';
    if (line.line_type === 'Addition') prefix = '+';
    else if (line.line_type === 'Deletion') prefix = '-';
    patch += prefix + line.content + '\n';
  });
  return patch;
}

function renderDiff(diffs, filePath) {
  if (!diffs || diffs.length === 0) {
    renderEmptyState();
    return;
  }

  const diff = diffs[0];
  el.diffFilename.textContent = filePath;
  el.statAdd.textContent = `+${diff.hunks.reduce((acc, h) => acc + h.lines.filter(l => l.line_type === 'Addition').length, 0)}`;
  el.statDel.textContent = `-${diff.hunks.reduce((acc, h) => acc + h.lines.filter(l => l.line_type === 'Deletion').length, 0)}`;

  if (diff.is_binary) {
    el.diffContent.innerHTML = `
      <div class="empty-state">
        <h2>Binary file not shown</h2>
        <p>This binary file cannot be viewed in unified diff format.</p>
      </div>
    `;
    return;
  }

  const isChangesMode = (state.activeTab === 'changes' && state.selectedFile);

  let html = '<div class="diff-table">';
  diff.hunks.forEach((hunk, hunkIdx) => {
    html += `
      <div class="diff-row hunk diff-hunk-header">
        <div class="diff-line-content">${escapeHtml(hunk.header)}</div>
        ${isChangesMode ? `
          <div class="diff-hunk-actions">
            <button class="btn-hunk-action hunk-stage-btn" data-hunk-idx="${hunkIdx}">Stage Hunk</button>
            <button class="btn-hunk-action danger hunk-discard-btn" data-hunk-idx="${hunkIdx}">Discard Hunk</button>
          </div>
        ` : ''}
      </div>
    `;

    hunk.lines.forEach(line => {
      let rowClass = '';
      let prefix = ' ';
      if (line.line_type === 'Addition') {
        rowClass = 'add';
        prefix = '+';
      } else if (line.line_type === 'Deletion') {
        rowClass = 'del';
        prefix = '-';
      }

      const oldNo = line.old_lineno != null ? line.old_lineno : '';
      const newNo = line.new_lineno != null ? line.new_lineno : '';

      html += `
        <div class="diff-row ${rowClass}">
          <div class="diff-gutter">
            <span>${oldNo}</span>
            <span>${newNo}</span>
          </div>
          <div class="diff-line-content">${prefix} ${escapeHtml(line.content)}</div>
        </div>
      `;
    });
  });
  html += '</div>';

  el.diffContent.innerHTML = html;

  if (isChangesMode) {
    el.diffContent.querySelectorAll('.hunk-stage-btn').forEach(btn => {
      btn.addEventListener('click', async (e) => {
        e.stopPropagation();
        const idx = parseInt(btn.getAttribute('data-hunk-idx'), 10);
        const hunk = diff.hunks[idx];
        const patch = buildHunkPatch(state.selectedFile, hunk);
        try {
          await invoke('apply_patch', { patch, cached: true, reverse: false });
          showToast('Staged hunk');
          await Promise.all([refreshStatus(), refreshRepoInfo()]);
          loadDiff(state.selectedFile);
        } catch (err) {
          showToast(`Stage hunk failed: ${err}`, true);
        }
      });
    });

    el.diffContent.querySelectorAll('.hunk-discard-btn').forEach(btn => {
      btn.addEventListener('click', async (e) => {
        e.stopPropagation();
        if (!confirm('Are you sure you want to discard this hunk?')) return;
        const idx = parseInt(btn.getAttribute('data-hunk-idx'), 10);
        const hunk = diff.hunks[idx];
        const patch = buildHunkPatch(state.selectedFile, hunk);
        try {
          await invoke('apply_patch', { patch, cached: false, reverse: true });
          showToast('Discarded hunk');
          await refreshStatus();
          loadDiff(state.selectedFile);
        } catch (err) {
          showToast(`Discard hunk failed: ${err}`, true);
        }
      });
    });
  }
}

function renderEmptyState() {
  el.diffFilename.textContent = 'No local changes';
  el.statAdd.textContent = '+0';
  el.statDel.textContent = '-0';

  el.diffContent.innerHTML = `
    <div class="empty-state">
      <svg class="empty-icon" viewBox="0 0 16 16" width="48" height="48" fill="currentColor">
        <path d="M1.5 3.25a2.25 2.25 0 1 1 3 2.122v5.256a2.251 2.251 0 1 1-1.5 0V5.372A2.25 2.25 0 0 1 1.5 3.25Zm5.677-.177L9.5 5.396V2.75a.75.75 0 0 1 1.5 0v4.5a.75.75 0 0 1-.75.75h-4.5a.75.75 0 0 1 0-1.5h2.646L7.177 3.78a.25.25 0 0 1 0-.354l.001-.001a.25.25 0 0 1 .354 0ZM14.5 12.75a2.25 2.25 0 1 1-4.5 0 2.25 2.25 0 0 1 4.5 0Zm-2.25.75a.75.75 0 1 0 0-1.5.75.75 0 0 0 0 1.5Z"/>
      </svg>
      <h2>No local changes</h2>
      <p>There are no uncommitted changes in this repository. Here are some friendly suggestions for what to do next:</p>
      <div class="empty-shortcuts">
        <button class="shortcut-btn" id="empty-open-editor">
          <span>Open in default editor</span>
        </button>
        <button class="shortcut-btn" id="empty-open-terminal">
          <span>Open in Terminal</span>
        </button>
        <button class="shortcut-btn" id="empty-reveal-finder">
          <span>Show in Finder</span>
        </button>
      </div>
    </div>
  `;

  document.getElementById('empty-open-editor')?.addEventListener('click', () => invoke('open_in_editor'));
  document.getElementById('empty-open-terminal')?.addEventListener('click', () => invoke('open_in_terminal'));
  document.getElementById('empty-reveal-finder')?.addEventListener('click', () => invoke('reveal_in_finder'));
}

function renderCommitList(commits, filter = '') {
  el.commitList.innerHTML = '';
  const filtered = filter
    ? commits.filter(c =>
        (c.summary && c.summary.toLowerCase().includes(filter.toLowerCase())) ||
        (c.author && c.author.name && c.author.name.toLowerCase().includes(filter.toLowerCase())) ||
        (c.sha && c.sha.toLowerCase().startsWith(filter.toLowerCase())) ||
        (c.short_sha && c.short_sha.toLowerCase().startsWith(filter.toLowerCase()))
      )
    : commits;

  if (filtered.length === 0) {
    el.commitList.innerHTML = '<div style="padding: 24px; text-align: center; color: var(--text-muted); font-size: 12px;">No commits found</div>';
    return;
  }

  filtered.forEach(commit => {
    const isHead = (commit.sha === state.commits[0]?.sha);
    const div = document.createElement('div');
    div.className = `commit-item ${state.selectedCommit === commit.sha ? 'selected' : ''}`;

    div.innerHTML = `
      <div class="commit-item-top">
        <div class="commit-summary-line">${escapeHtml(commit.summary)}</div>
        <button class="commit-revert-quick-btn" title="Revert this commit">Revert</button>
      </div>
      <div class="commit-meta-line">
        <span>${escapeHtml(commit.author?.name || '')}</span>
        <span>•</span>
        <span>${escapeHtml(commit.date || '')}</span>
        <span class="commit-sha-pill">${commit.short_sha}</span>
      </div>
    `;

    const revertBtn = div.querySelector('.commit-revert-quick-btn');
    if (revertBtn) {
      revertBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        confirmAndRevertCommit(commit);
      });
    }

    div.addEventListener('click', () => {
      selectCommit(commit, isHead);
    });

    el.commitList.appendChild(div);
  });
}

async function selectCommit(commit, isHead = false) {
  state.selectedCommit = commit.sha;
  const items = el.commitList.querySelectorAll('.commit-item');
  items.forEach(it => {
    const pill = it.querySelector('.commit-sha-pill');
    it.classList.toggle('selected', pill && pill.textContent === commit.short_sha);
  });

  // Populate commit header
  if (el.commitHeaderTitle) el.commitHeaderTitle.textContent = commit.summary;
  if (el.commitHeaderAuthor) el.commitHeaderAuthor.textContent = `${commit.author?.name || ''} <${commit.author?.email || ''}>`;
  if (el.commitHeaderDate) el.commitHeaderDate.textContent = commit.date || '';
  if (el.commitHeaderSha) {
    el.commitHeaderSha.textContent = commit.short_sha;
    el.commitHeaderSha.title = `Full SHA: ${commit.sha} (Click to copy)`;
  }

  if (el.commitHeaderDesc) {
    if (commit.body && commit.body.trim()) {
      el.commitHeaderDesc.textContent = commit.body.trim();
      el.commitHeaderDesc.style.display = 'block';
    } else {
      el.commitHeaderDesc.textContent = '';
      el.commitHeaderDesc.style.display = 'none';
    }
  }

  // Header display logic
  if (state.activeTab === 'history') {
    if (el.commitHeader) el.commitHeader.style.display = 'flex';
    if (el.commitFilesBar) el.commitFilesBar.style.display = 'flex';
    if (el.diffHeader) el.diffHeader.style.display = 'none';
  }

  // Show Undo button only on HEAD commit
  if (el.historyUndoBtn) {
    el.historyUndoBtn.style.display = isHead ? 'inline-flex' : 'none';
  }

  // Load changed files in this commit
  state.selectedCommitFile = null;
  try {
    const files = await invoke('get_commit_files', { sha: commit.sha });
    state.commitFiles = files || [];
    renderCommitFilesBar(commit);
  } catch (err) {
    console.error('Failed to load commit files:', err);
  }

  await loadCommitDiff(commit.sha, state.selectedCommitFile);
}

function renderCommitFilesBar(commit) {
  if (!el.commitFilesBar || !el.commitFilesList) return;
  el.commitFilesList.innerHTML = '';
  const count = state.commitFiles.length;
  el.commitFilesSummary.textContent = `${count} ${count === 1 ? 'file' : 'files'} changed`;

  // "All Files" tab
  const allTab = document.createElement('div');
  allTab.className = `commit-file-tab ${state.selectedCommitFile === null ? 'selected' : ''}`;
  allTab.innerHTML = `<span>All Files</span>`;
  allTab.addEventListener('click', async () => {
    state.selectedCommitFile = null;
    el.commitFilesList.querySelectorAll('.commit-file-tab').forEach(t => t.classList.remove('selected'));
    allTab.classList.add('selected');
    await loadCommitDiff(commit.sha, null);
  });
  el.commitFilesList.appendChild(allTab);

  state.commitFiles.forEach(file => {
    const tab = document.createElement('div');
    const isSelected = state.selectedCommitFile === file.path;
    tab.className = `commit-file-tab ${isSelected ? 'selected' : ''}`;

    let statusClass = `status-${file.status}`;
    const baseName = file.path.split('/').pop() || file.path;

    tab.innerHTML = `
      <span class="status-badge ${statusClass}">${file.status}</span>
      <span title="${escapeHtml(file.path)}">${escapeHtml(baseName)}</span>
      <span style="color: var(--diff-add-text); font-size: 10.5px;">+${file.additions}</span>
      <span style="color: var(--diff-del-text); font-size: 10.5px;">-${file.deletions}</span>
    `;

    tab.addEventListener('click', async () => {
      state.selectedCommitFile = file.path;
      el.commitFilesList.querySelectorAll('.commit-file-tab').forEach(t => t.classList.remove('selected'));
      tab.classList.add('selected');
      await loadCommitDiff(commit.sha, file.path);
    });

    el.commitFilesList.appendChild(tab);
  });
}

async function loadCommitDiff(sha, filePath = null) {
  try {
    const diffs = await invoke('get_commit_diff', { sha, filePath });
    renderDiff(diffs, filePath ? `Commit: ${sha.substring(0, 7)} — ${filePath}` : `Commit: ${sha.substring(0, 7)}`);
  } catch (err) {
    console.error('Failed to load commit diff:', err);
    el.diffContent.innerHTML = `<div class="empty-state"><p>Error loading commit diff: ${err}</p></div>`;
  }
}

async function confirmAndRevertCommit(commit) {
  const shortSha = commit.short_sha || commit.sha.substring(0, 7);
  const msg = `Are you sure you want to revert commit ${shortSha} ("${commit.summary}")?\n\nThis will create a new commit that inverts the changes.`;
  if (!confirm(msg)) return;

  try {
    showToast(`Reverting ${shortSha}…`);
    const newSha = await invoke('revert_commit', { sha: commit.sha });
    showToast(`Reverted ${shortSha} (new commit: ${newSha.substring(0, 7)})`);
    await Promise.all([
      refreshRepoInfo(),
      refreshStatus(),
      loadCommits()
    ]);
  } catch (err) {
    showToast(`Revert failed: ${err}`, true);
  }
}

function renderBranchList(branches, filter = '') {
  el.modalBranchList.innerHTML = '';
  const filtered = branches.filter(b => b.name.toLowerCase().includes(filter.toLowerCase()));

  filtered.forEach(branch => {
    const row = document.createElement('div');
    row.className = `branch-row ${branch.is_current ? 'current' : ''}`;
    row.innerHTML = `
      <span>${branch.is_current ? '✓ ' : ''}${escapeHtml(branch.name)}</span>
      <span style="font-size: 11px; color: var(--text-dim);">${branch.is_remote ? 'Remote' : 'Local'}</span>
    `;

    row.addEventListener('click', async () => {
      try {
        await invoke('checkout_branch', { name: branch.name });
        showToast(`Switched to ${branch.name}`);
        el.branchModal.classList.remove('open');
        await refreshRepoInfo();
        await refreshStatus();
        await loadCommits();
      } catch (err) {
        showToast(`Checkout failed: ${err}`, true);
      }
    });

    el.modalBranchList.appendChild(row);
  });
}

async function openMergeModal() {
  const branches = await invoke('get_branches');
  state.branches = branches || [];
  const curr = state.status?.branch || 'main';
  el.mergeTargetBranch.textContent = curr;
  el.mergeTargetBranchText.textContent = curr;
  state.selectedMergeBranch = null;
  el.mergeConfirmBtn.disabled = true;
  el.mergeConfirmBtn.textContent = 'Merge Branch';
  el.mergeSearchInput.value = '';
  renderMergeBranchList(state.branches, curr, '');
  el.mergeModal.classList.add('open');
  el.mergeSearchInput.focus();
}

function renderMergeBranchList(branches, currentBranch, filter = '') {
  el.modalMergeBranchList.innerHTML = '';
  const filtered = branches.filter(b => 
    !b.is_current &&
    b.name !== currentBranch &&
    b.name.toLowerCase().includes(filter.toLowerCase())
  );

  if (filtered.length === 0) {
    el.modalMergeBranchList.innerHTML = '<div style="padding: 16px; text-align: center; color: var(--text-muted); font-size: 12px;">No other branches available to merge</div>';
    return;
  }

  filtered.forEach(branch => {
    const row = document.createElement('div');
    row.className = `modal-branch-row ${state.selectedMergeBranch === branch.name ? 'current' : ''}`;
    row.innerHTML = `
      <div style="display: flex; align-items: center; gap: 8px;">
        <svg class="octicon" viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
          <path d="M9.5 3.25a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.492 2.492 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25Zm-6 0a.75.75 0 1 0 1.5 0 .75.75 0 0 0-1.5 0Zm8.25.75a.75.75 0 1 0 0-1.5.75.75 0 0 0 0 1.5ZM4.25 12a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5Z"/>
        </svg>
        <span style="font-weight: 500;">${escapeHtml(branch.name)}</span>
      </div>
      <span style="font-size: 11px; color: var(--text-dim);">${branch.is_remote ? 'Remote' : 'Local'}</span>
    `;

    row.addEventListener('click', () => {
      state.selectedMergeBranch = branch.name;
      const allRows = el.modalMergeBranchList.querySelectorAll('.modal-branch-row');
      allRows.forEach(r => r.classList.remove('current'));
      row.classList.add('current');
      el.mergeConfirmBtn.disabled = false;
      el.mergeConfirmBtn.textContent = `Merge ${branch.name} into ${currentBranch}`;
    });

    el.modalMergeBranchList.appendChild(row);
  });
}

function openSettingsModal() {
  el.settingsUserName.value = localStorage.getItem('desktop_user_name') || 'Brandon Hubbard';
  el.settingsUserEmail.value = localStorage.getItem('desktop_user_email') || 'bhubbard@users.noreply.github.com';
  el.settingsEditorSelect.value = localStorage.getItem('desktop_editor') || 'zed';
  el.settingsModal.classList.add('open');
}

function renderRepoList(repos, filter = '') {
  el.modalRepoList.innerHTML = '';
  const term = (filter || '').toLowerCase().trim();
  const filtered = (repos || []).filter(r => r.name.toLowerCase().includes(term) || r.path.toLowerCase().includes(term));

  if (filtered.length === 0) {
    el.modalRepoList.innerHTML = '<div style="padding: 20px; text-align: center; color: var(--text-muted);">No matching repositories</div>';
    return;
  }

  filtered.forEach(repo => {
    const row = document.createElement('div');
    row.className = `modal-branch-row ${repo.is_current ? 'current' : ''}`;
    row.innerHTML = `
      <div style="display: flex; align-items: center; gap: 10px; overflow: hidden; flex: 1;">
        <svg class="octicon" viewBox="0 0 16 16" width="16" height="16" fill="currentColor">
          <path d="M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-.714 1.7.75.75 0 1 1-1.072 1.05A2.495 2.495 0 0 1 2 11.5Zm10.5-1h-8a1 1 0 0 0-1 1v6.708A2.486 2.486 0 0 1 4.5 9h8ZM5 12.25a.25.25 0 0 1 .25-.25H12v1.5H5.25a.25.25 0 0 1-.25-.25Z"/>
        </svg>
        <div style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap; line-height: 1.35;">
          <div style="font-weight: 600;">${escapeHtml(repo.name)} ${repo.is_current ? '<span style="color: var(--accent); font-size: 11px; font-weight: normal;">(current)</span>' : ''}</div>
          <div style="font-size: 11px; color: var(--text-dim); overflow: hidden; text-overflow: ellipsis;">${escapeHtml(repo.path)}</div>
        </div>
      </div>
      <div style="display: flex; align-items: center; gap: 8px;">
        ${repo.is_current ? '<span style="color: var(--btn-commit-bg); font-weight: 700;">✓</span>' : '<button class="btn-subtle" style="font-size: 11px; padding: 2px 8px;">Switch</button>'}
        ${!repo.is_current ? '<button class="btn-repo-remove" title="Remove from list">✕</button>' : ''}
      </div>
    `;

    const removeBtn = row.querySelector('.btn-repo-remove');
    if (removeBtn) {
      removeBtn.addEventListener('click', async (e) => {
        e.stopPropagation();
        try {
          await invoke('remove_repository', { path: repo.path });
          showToast(`Removed ${repo.name} from list`);
          state.repositories = (state.repositories || []).filter(r => r.path !== repo.path);
          renderRepoList(state.repositories, el.repoSearchInput ? el.repoSearchInput.value : '');
        } catch (err) {
          showToast(`Failed to remove repository: ${err}`, true);
        }
      });
    }

    row.addEventListener('click', async () => {
      if (repo.is_current) {
        el.repoModal.classList.remove('open');
        return;
      }
      try {
        await invoke('switch_repository', { newPath: repo.path });
        showToast(`Switched to repository ${repo.name}`);
        el.repoModal.classList.remove('open');
        await refreshRepoInfo();
        await refreshStatus();
        await loadCommits();
      } catch (err) {
        showToast(`Switch failed: ${err}`, true);
      }
    });

    el.modalRepoList.appendChild(row);
  });
}

// Event Listeners
function setupEventListeners() {
  // Tabs
  el.tabChanges.addEventListener('click', () => {
    state.activeTab = 'changes';
    el.tabChanges.classList.add('active');
    el.tabHistory.classList.remove('active');
    el.viewChanges.classList.add('active');
    el.viewHistory.classList.remove('active');
    if (el.commitHeader) el.commitHeader.style.display = 'none';
    if (el.commitFilesBar) el.commitFilesBar.style.display = 'none';
    if (el.diffHeader) el.diffHeader.style.display = 'flex';
    if (state.selectedFile) {
      loadDiff(state.selectedFile);
    } else {
      renderEmptyState();
    }
  });

  el.tabHistory.addEventListener('click', () => {
    state.activeTab = 'history';
    el.tabHistory.classList.add('active');
    el.tabChanges.classList.remove('active');
    el.viewHistory.classList.add('active');
    el.viewChanges.classList.remove('active');
    if (el.diffHeader) el.diffHeader.style.display = 'none';
    if (el.commitFilesBar && state.selectedCommit) el.commitFilesBar.style.display = 'flex';
    if (state.selectedCommit) {
      const commit = state.commits.find(c => c.sha === state.selectedCommit);
      if (commit) {
        selectCommit(commit, commit.sha === state.commits[0]?.sha);
      }
    } else if (state.commits.length > 0) {
      selectCommit(state.commits[0], true);
    }
  });

  // Stash Banner Actions
  if (el.stashRestoreBtn) {
    el.stashRestoreBtn.addEventListener('click', async () => {
      try {
        const idx = parseInt(el.stashBanner.dataset.index || '0', 10);
        await invoke('stash_pop', { index: idx });
        showToast('Restored stashed changes');
        await Promise.all([
          refreshStatus(),
          refreshRepoInfo(),
          refreshStash()
        ]);
      } catch (err) {
        showToast(`Stash restore failed: ${err}`, true);
      }
    });
  }

  if (el.stashDiscardBtn) {
    el.stashDiscardBtn.addEventListener('click', async () => {
      const idx = parseInt(el.stashBanner.dataset.index || '0', 10);
      if (confirm('Are you sure you want to discard these stashed changes?')) {
        try {
          await invoke('stash_drop', { index: idx });
          showToast('Discarded stash');
          await refreshStash();
        } catch (err) {
          showToast(`Stash discard failed: ${err}`, true);
        }
      }
    });
  }

  // History Filter
  if (el.historyFilter) {
    el.historyFilter.addEventListener('input', (e) => {
      renderCommitList(state.commits, e.target.value.trim());
    });
  }

  // History Header Actions
  const copySelectedSha = () => {
    if (!state.selectedCommit) return;
    navigator.clipboard.writeText(state.selectedCommit);
    showToast('Copied full SHA to clipboard');
  };
  if (el.copyShaBtn) el.copyShaBtn.addEventListener('click', copySelectedSha);
  if (el.commitHeaderSha) el.commitHeaderSha.addEventListener('click', copySelectedSha);

  if (el.historyUndoBtn) {
    el.historyUndoBtn.addEventListener('click', async () => {
      try {
        await invoke('undo_commit');
        showToast('Undid latest commit (soft reset)');
        await Promise.all([
          refreshRepoInfo(),
          refreshStatus(),
          loadCommits()
        ]);
        el.tabChanges.click();
      } catch (err) {
        showToast(`Undo error: ${err}`, true);
      }
    });
  }

  if (el.historyRevertBtn) {
    el.historyRevertBtn.addEventListener('click', () => {
      if (!state.selectedCommit) {
        showToast('Select a commit in History to revert', true);
        return;
      }
      const commit = state.commits.find(c => c.sha === state.selectedCommit);
      if (commit) {
        confirmAndRevertCommit(commit);
      }
    });
  }

  // Select All Checkbox
  el.selectAllCheckbox.addEventListener('change', async (e) => {
    if (e.target.checked) {
      await invoke('stage_all');
    } else {
      await invoke('unstage_all');
    }
    await refreshStatus();
  });

  // Commit
  el.commitBtn.addEventListener('click', async () => {
    const summary = el.commitSummary.value.trim();
    const description = el.commitDescription.value.trim() || null;

    if (!summary) {
      showToast('Commit summary is required', true);
      el.commitSummary.focus();
      return;
    }

    const origBtnHtml = el.commitBtn.innerHTML;
    el.commitBtn.disabled = true;
    el.commitBtn.innerHTML = `
      <span class="commit-btn-spinner" style="display:inline-block; width:12px; height:12px; border:2px solid rgba(255,255,255,0.3); border-top-color:#fff; border-radius:50%; animation:spin 0.8s linear infinite; margin-right:6px; vertical-align:middle;"></span>
      <span>Committing…</span>
    `;

    try {
      const sha = await invoke('commit', { summary, description, coAuthors: [] });
      showToast(`Committed ${sha.substring(0, 7)}`);
      el.commitSummary.value = '';
      el.commitDescription.value = '';
      await Promise.all([
        refreshStatus(),
        refreshRepoInfo(),
        loadCommits()
      ]);
    } catch (err) {
      showToast(`Commit error: ${err}`, true);
    } finally {
      el.commitBtn.disabled = false;
      el.commitBtn.innerHTML = origBtnHtml;
      const branchLabel = el.commitBtn.querySelector('#commit-branch-label');
      if (branchLabel && el.branchName) {
        branchLabel.textContent = el.branchName.textContent;
      }
    }
  });

  // Undo Commit (from Changes tab)
  el.undoBtn.addEventListener('click', async () => {
    try {
      await invoke('undo_commit');
      showToast('Undid latest commit (soft reset)');
      await Promise.all([
        refreshRepoInfo(),
        refreshStatus(),
        loadCommits()
      ]);
    } catch (err) {
      showToast(`Undo error: ${err}`, true);
    }
  });

  // Sync / Fetch Button
  el.syncBtn.addEventListener('click', async () => {
    if (state.isSyncing) return;
    state.isSyncing = true;
    el.syncSpinner.style.animation = 'spin 1s linear infinite';
    try {
      const res = await invoke('sync_remote');
      showToast(res || 'Synced with remote');
      await refreshRepoInfo();
      await refreshStatus();
      await loadCommits();
    } catch (err) {
      showToast(`Sync failed: ${err}`, true);
    } finally {
      state.isSyncing = false;
      el.syncSpinner.style.animation = 'none';
    }
  });

  // Quick Action Buttons
  el.openEditorBtn.addEventListener('click', () => invoke('open_in_editor', { path: state.selectedFile }));
  el.revealFinderBtn.addEventListener('click', () => invoke('reveal_in_finder', { path: state.selectedFile }));
  el.discardFileBtn.addEventListener('click', async () => {
    if (!state.selectedFile) return;
    if (confirm(`Are you sure you want to discard changes in ${state.selectedFile}?`)) {
      await invoke('discard_file', { path: state.selectedFile });
      showToast(`Discarded ${state.selectedFile}`);
      await refreshStatus();
    }
  });
  el.terminalBtn.addEventListener('click', () => invoke('open_in_terminal'));

  // Branch Switcher Modal
  el.branchBtn.addEventListener('click', async () => {
    const branches = await invoke('get_branches');
    state.branches = branches || [];
    renderBranchList(state.branches);
    el.branchSearchInput.value = '';
    el.branchModal.classList.add('open');
    el.branchSearchInput.focus();
  });

  el.branchModalClose.addEventListener('click', () => {
    el.branchModal.classList.remove('open');
  });

  el.branchSearchInput.addEventListener('input', (e) => {
    renderBranchList(state.branches, e.target.value);
  });

  el.createBranchBtn.addEventListener('click', async () => {
    const name = el.newBranchName.value.trim();
    if (!name) return;
    try {
      await invoke('create_branch', { name });
      showToast(`Created & switched to ${name}`);
      el.newBranchName.value = '';
      el.branchModal.classList.remove('open');
      await refreshRepoInfo();
      await refreshStatus();
    } catch (err) {
      showToast(`Create branch failed: ${err}`, true);
    }
  });

  // Pull Requests Modal
  el.prsBtn.addEventListener('click', async () => {
    el.prList.innerHTML = '<div style="padding: 20px; text-align: center;">Loading pull requests...</div>';
    el.prModal.classList.add('open');
    try {
      const prs = await invoke('get_github_prs');
      state.pullRequests = prs || [];
      if (state.pullRequests.length === 0) {
        el.prList.innerHTML = '<div style="padding: 24px; text-align: center; color: var(--text-muted);">No open pull requests</div>';
        return;
      }
      el.prList.innerHTML = '';
      state.pullRequests.forEach(pr => {
        const row = document.createElement('div');
        row.className = 'pr-row';
        row.innerHTML = `
          <div>
            <strong style="color: var(--accent);">#${pr.number}</strong> ${escapeHtml(pr.title)}
            <div style="font-size: 11px; color: var(--text-dim);">by @${escapeHtml(pr.author)} (${escapeHtml(pr.head_branch)})</div>
          </div>
          <button class="btn-subtle" style="font-weight: 600;">Checkout</button>
        `;
        row.querySelector('button').addEventListener('click', async () => {
          try {
            await invoke('checkout_pr', { number: pr.number });
            showToast(`Checked out PR #${pr.number}`);
            el.prModal.classList.remove('open');
            await refreshRepoInfo();
            await refreshStatus();
          } catch (err) {
            showToast(`Checkout PR failed: ${err}`, true);
          }
        });
        el.prList.appendChild(row);
      });
    } catch (err) {
      el.prList.innerHTML = `<div style="padding: 20px; color: var(--danger); text-align: center;">Error loading PRs: ${err}</div>`;
    }
  });

  el.prModalClose.addEventListener('click', () => {
    el.prModal.classList.remove('open');
  });

  // Repository Switcher Button on Header
  el.repoBtn.addEventListener('click', async () => {
    try {
      const repos = await invoke('get_repositories');
      state.repositories = repos || [];
      renderRepoList(state.repositories);
    } catch (err) {
      console.warn('Failed to load repositories:', err);
    }
    el.repoSearchInput.value = '';
    el.repoModal.classList.add('open');
    el.repoSearchInput.focus();
  });

  el.repoModalClose.addEventListener('click', () => el.repoModal.classList.remove('open'));
  el.repoSearchInput.addEventListener('input', (e) => renderRepoList(state.repositories, e.target.value));
  el.repoModalAddBtn.addEventListener('click', () => {
    el.repoModal.classList.remove('open');
    el.addRepoModal.classList.add('open');
  });
  el.repoModalNewBtn.addEventListener('click', () => {
    el.repoModal.classList.remove('open');
    el.newRepoModal.classList.add('open');
  });
  el.repoModalCloneBtn.addEventListener('click', () => {
    el.repoModal.classList.remove('open');
    el.cloneRepoModal.classList.add('open');
  });

  // New Repository Modal
  el.newRepoModalClose.addEventListener('click', () => el.newRepoModal.classList.remove('open'));
  el.newRepoCancelBtn.addEventListener('click', () => el.newRepoModal.classList.remove('open'));
  el.newRepoCreateBtn.addEventListener('click', async () => {
    const name = el.newRepoNameInput.value.trim();
    const parentPath = el.newRepoPathInput.value.trim();
    const initReadme = el.newRepoReadmeCheckbox.checked;
    if (!name || !parentPath) {
      showToast('Please provide both repository name and local path', true);
      return;
    }
    try {
      await invoke('init_repository', { name, parentPath, initReadme });
      showToast(`Created & opened ${name}`);
      el.newRepoModal.classList.remove('open');
      el.newRepoNameInput.value = '';
      await refreshRepoInfo();
      await refreshStatus();
      await loadCommits();
    } catch (err) {
      showToast(`Init failed: ${err}`, true);
    }
  });

  // Add Local Repository Modal
  el.addRepoModalClose.addEventListener('click', () => el.addRepoModal.classList.remove('open'));
  el.addRepoCancelBtn.addEventListener('click', () => el.addRepoModal.classList.remove('open'));
  el.addRepoConfirmBtn.addEventListener('click', async () => {
    const newPath = el.addRepoPathInput.value.trim();
    if (!newPath) return;
    try {
      await invoke('switch_repository', { newPath });
      showToast(`Switched repository`);
      el.addRepoModal.classList.remove('open');
      await refreshRepoInfo();
      await refreshStatus();
      await loadCommits();
    } catch (err) {
      showToast(`Add repository failed: ${err}`, true);
    }
  });

  // Clone Repository Modal
  el.cloneRepoModalClose.addEventListener('click', () => el.cloneRepoModal.classList.remove('open'));
  el.cloneRepoCancelBtn.addEventListener('click', () => el.cloneRepoModal.classList.remove('open'));
  el.cloneRepoConfirmBtn.addEventListener('click', async () => {
    const url = el.cloneRepoUrlInput.value.trim();
    const destination = el.cloneRepoDestInput.value.trim();
    if (!url || !destination) {
      showToast('Please specify repository URL and destination path', true);
      return;
    }
    showToast(`Cloning ${url}...`);
    try {
      await invoke('clone_repository', { url, destination });
      showToast(`Cloned successfully`);
      el.cloneRepoModal.classList.remove('open');
      await refreshRepoInfo();
      await refreshStatus();
      await loadCommits();
    } catch (err) {
      showToast(`Clone failed: ${err}`, true);
    }
  });

  // Merge Branch Modal
  if (el.mergeModalClose) el.mergeModalClose.addEventListener('click', () => el.mergeModal.classList.remove('open'));
  if (el.mergeCancelBtn) el.mergeCancelBtn.addEventListener('click', () => el.mergeModal.classList.remove('open'));
  if (el.mergeSearchInput) {
    el.mergeSearchInput.addEventListener('input', (e) => {
      renderMergeBranchList(state.branches, state.status?.branch || 'main', e.target.value);
    });
  }
  if (el.mergeConfirmBtn) {
    el.mergeConfirmBtn.addEventListener('click', async () => {
      if (!state.selectedMergeBranch) return;
      try {
        showToast(`Merging ${state.selectedMergeBranch}…`);
        const res = await invoke('merge_branch', { branch: state.selectedMergeBranch });
        showToast(res || `Merged ${state.selectedMergeBranch}`);
        el.mergeModal.classList.remove('open');
        await Promise.all([
          refreshRepoInfo(),
          refreshStatus(),
          loadCommits(),
          refreshStash()
        ]);
      } catch (err) {
        showToast(`Merge failed: ${err}`, true);
      }
    });
  }

  // Settings / Preferences Modal
  if (el.settingsModalClose) el.settingsModalClose.addEventListener('click', () => el.settingsModal.classList.remove('open'));
  if (el.settingsCancelBtn) el.settingsCancelBtn.addEventListener('click', () => el.settingsModal.classList.remove('open'));
  if (el.settingsSaveBtn) {
    el.settingsSaveBtn.addEventListener('click', () => {
      const name = el.settingsUserName.value.trim();
      const email = el.settingsUserEmail.value.trim();
      const editor = el.settingsEditorSelect.value;
      if (name) localStorage.setItem('desktop_user_name', name);
      if (email) localStorage.setItem('desktop_user_email', email);
      if (editor) localStorage.setItem('desktop_editor', editor);
      showToast('Settings saved');
      el.settingsModal.classList.remove('open');
    });
  }

  // About Modal
  el.aboutModalCloseBtn.addEventListener('click', () => el.aboutModal.classList.remove('open'));

  const allModals = [
    el.repoModal,
    el.branchModal,
    el.prModal,
    el.newRepoModal,
    el.addRepoModal,
    el.cloneRepoModal,
    el.mergeModal,
    el.settingsModal,
    el.aboutModal,
  ];

  // Close modals on Esc or Backdrop click
  window.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      allModals.forEach(m => m && m.classList.remove('open'));
    }
  });

  allModals.forEach(modal => {
    if (!modal) return;
    modal.addEventListener('click', (e) => {
      if (e.target === modal) {
        modal.classList.remove('open');
      }
    });
  });

  // Native Menu Events Listener
  setupMenuEventListeners();

  // Enable window dragging across titlebar and drag regions
  setupWindowDragging();
}

function setupMenuEventListeners() {
  const listenFn = (window.__TAURI__ && window.__TAURI__.event && window.__TAURI__.event.listen)
    || (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.listen);
  if (typeof listenFn === 'function') {
    listenFn('menu-event', (event) => {
      handleMenuCommand(event.payload);
    });
  }
}

async function handleMenuCommand(id) {
  switch (id) {
    case 'new-repository':
      el.newRepoModal.classList.add('open');
      el.newRepoNameInput.focus();
      break;
    case 'add-local-repository':
      el.addRepoModal.classList.add('open');
      el.addRepoPathInput.focus();
      break;
    case 'clone-repository':
      el.cloneRepoModal.classList.add('open');
      el.cloneRepoUrlInput.focus();
      break;
    case 'preferences':
      openSettingsModal();
      break;
    case 'show-changes':
      el.tabChanges.click();
      break;
    case 'show-history':
      el.tabHistory.click();
      break;
    case 'show-repository-list':
      el.addRepoModal.classList.add('open');
      break;
    case 'show-branches-list':
      el.branchBtn.click();
      break;
    case 'create-branch':
      el.branchBtn.click();
      setTimeout(() => el.newBranchName.focus(), 100);
      break;
    case 'merge-into-current-branch':
      openMergeModal();
      break;
    case 'push':
    case 'pull':
    case 'fetch':
      el.syncBtn.click();
      break;
    case 'view-repository-on-github':
      try {
        const summary = await invoke('get_repo_summary');
        const url = `https://github.com/bhubbard/${summary.name}`;
        window.open(url, '_blank');
      } catch {
        window.open('https://github.com/bhubbard/desktop-rs', '_blank');
      }
      break;
    case 'open-in-shell':
      await invoke('open_in_terminal');
      break;
    case 'open-working-directory':
      await invoke('reveal_in_finder');
      break;
    case 'open-external-editor':
      await invoke('open_in_editor');
      break;
    case 'discard-all-changes':
      if (confirm('Are you sure you want to discard all changes in this repository?')) {
        for (const f of state.status.files) {
          await invoke('discard_file', { path: f.path });
        }
        await refreshStatus();
        showToast('Discarded all changes');
      }
      break;
    case 'stash-all-changes':
      try {
        await invoke('stash_save', { message: null, keepIndex: false });
        showToast('Stashed all changes');
        await Promise.all([
          refreshStatus(),
          refreshRepoInfo(),
          refreshStash()
        ]);
      } catch (e) {
        showToast(`Stash error: ${e}`, true);
      }
      break;
    case 'revert-commit':
      if (state.activeTab !== 'history') {
        el.tabHistory.click();
      }
      setTimeout(() => {
        if (el.historyRevertBtn) {
          el.historyRevertBtn.click();
        }
      }, 50);
      break;
    case 'compare-on-github':
    case 'create-pull-request':
      el.prsBtn.click();
      break;
    case 'reload-window':
      window.location.reload();
      break;
    case 'report-issue':
      window.open('https://github.com/bhubbard/desktop-rs/issues/new', '_blank');
      break;
    case 'show-docs':
      window.open('https://github.com/bhubbard/desktop-rs', '_blank');
      break;
    case 'about':
      el.aboutModal.classList.add('open');
      break;
  }
}

function setupWindowDragging() {
  // Native window dragging on [data-tauri-drag-region] is handled by Tauri's core runtime.
  // Add macOS titlebar double-click to toggle maximize:
  document.addEventListener('dblclick', async (e) => {
    if (e.button !== 0) return;
    const isInteractive = e.target.closest(
      'button, input, textarea, a, select, [role="button"], .modal-content'
    );
    if (isInteractive) return;

    const isDragArea = e.target.closest('[data-tauri-drag-region]');
    if (isDragArea) {
      try {
        if (window.__TAURI__ && window.__TAURI__.window) {
          await window.__TAURI__.window.getCurrentWindow().toggleMaximize();
        }
      } catch (err) {
        console.error('Toggle maximize failed:', err);
      }
    }
  });
}

function escapeHtml(str) {
  if (!str) return '';
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

window.addEventListener('DOMContentLoaded', init);
