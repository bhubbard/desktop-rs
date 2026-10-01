// GitHub Desktop - Pure Rust + Tauri Frontend Engine
const invoke = window.__TAURI__ ? window.__TAURI__.core.invoke : async (cmd, args) => {
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
  pullRequests: [],
  isSyncing: false,
};

// DOM References
const el = {
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

  tabChanges: document.getElementById('tab-changes'),
  tabHistory: document.getElementById('tab-history'),
  viewChanges: document.getElementById('view-changes'),
  viewHistory: document.getElementById('view-history'),
  changesCountBadge: document.getElementById('changes-count'),
  changesSummaryCount: document.getElementById('changes-summary-count'),
  selectAllCheckbox: document.getElementById('select-all-checkbox'),
  fileList: document.getElementById('file-list'),

  commitSummary: document.getElementById('commit-summary'),
  commitDescription: document.getElementById('commit-description'),
  commitBtn: document.getElementById('commit-btn'),
  commitBranchLabel: document.getElementById('commit-branch-label'),
  undoBtn: document.getElementById('undo-btn'),

  historyFilter: document.getElementById('history-filter'),
  commitList: document.getElementById('commit-list'),

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

  aboutModal: document.getElementById('about-modal'),
  aboutModalCloseBtn: document.getElementById('about-modal-close-btn'),

  toast: document.getElementById('toast'),
};

// Initialization
async function init() {
  setupEventListeners();
  await refreshRepoInfo();
  await refreshStatus();
  await loadCommits();
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
    renderCommitList(state.commits);
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

  let html = '<div class="diff-table">';
  diff.hunks.forEach(hunk => {
    html += `<div class="diff-row hunk"><div class="diff-line-content">${escapeHtml(hunk.header)}</div></div>`;

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

function renderCommitList(commits) {
  el.commitList.innerHTML = '';
  commits.forEach(commit => {
    const div = document.createElement('div');
    div.className = `commit-item ${state.selectedCommit === commit.sha ? 'selected' : ''}`;

    div.innerHTML = `
      <div class="commit-summary-line">${escapeHtml(commit.summary)}</div>
      <div class="commit-meta-line">
        <span>${escapeHtml(commit.author.name)}</span>
        <span>•</span>
        <span>${escapeHtml(commit.date)}</span>
        <span class="commit-sha-pill">${commit.short_sha}</span>
      </div>
    `;

    div.addEventListener('click', async () => {
      state.selectedCommit = commit.sha;
      const items = el.commitList.querySelectorAll('.commit-item');
      items.forEach(it => it.classList.remove('selected'));
      div.classList.add('selected');

      const diffs = await invoke('get_commit_diff', { sha: commit.sha });
      renderDiff(diffs, `Commit: ${commit.short_sha} - ${commit.summary}`);
    });

    el.commitList.appendChild(div);
  });
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

// Event Listeners
function setupEventListeners() {
  // Tabs
  el.tabChanges.addEventListener('click', () => {
    state.activeTab = 'changes';
    el.tabChanges.classList.add('active');
    el.tabHistory.classList.remove('active');
    el.viewChanges.classList.add('active');
    el.viewHistory.classList.remove('active');
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
  });

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

    try {
      const sha = await invoke('commit', { summary, description, coAuthors: [] });
      showToast(`Committed ${sha.substring(0, 7)}`);
      el.commitSummary.value = '';
      el.commitDescription.value = '';
      await refreshRepoInfo();
      await refreshStatus();
      await loadCommits();
    } catch (err) {
      showToast(`Commit error: ${err}`, true);
    }
  });

  // Undo Commit
  el.undoBtn.addEventListener('click', async () => {
    try {
      await invoke('undo_commit');
      showToast('Undid latest commit (soft reset)');
      await refreshRepoInfo();
      await refreshStatus();
      await loadCommits();
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
  el.repoBtn.addEventListener('click', () => {
    el.addRepoModal.classList.add('open');
    if (state.status && state.status.path) {
      el.addRepoPathInput.value = state.status.path;
    }
    el.addRepoPathInput.focus();
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

  // About Modal
  el.aboutModalCloseBtn.addEventListener('click', () => el.aboutModal.classList.remove('open'));

  const allModals = [
    el.branchModal,
    el.prModal,
    el.newRepoModal,
    el.addRepoModal,
    el.cloneRepoModal,
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
  if (window.__TAURI__ && window.__TAURI__.event) {
    window.__TAURI__.event.listen('menu-event', (event) => {
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
      showToast('Settings: Git user & credentials auto-loaded from local Git keychain');
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
        await invoke('stage_all');
        showToast('Stashed changes');
        await refreshStatus();
      } catch (e) {
        showToast(`Stash error: ${e}`, true);
      }
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
  document.addEventListener('mousedown', (e) => {
    if (e.button !== 0) return;

    // Do not initiate drag if interacting with controls
    const isInteractive = e.target.closest(
      'button, input, textarea, a, select, [role="button"], .modal-content, [data-no-drag]'
    );
    if (isInteractive) return;

    // If clicking on header, drag spaces, traffic lights spacer, or data-tauri-drag-region
    const isDragArea = e.target.closest(
      '.app-header, [data-tauri-drag-region], .header-traffic-lights-spacer, .header-drag-space'
    );
    if (isDragArea) {
      if (window.__TAURI__ && window.__TAURI__.window) {
        window.__TAURI__.window.getCurrentWindow().startDragging();
      } else {
        invoke('start_dragging').catch(() => {});
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
