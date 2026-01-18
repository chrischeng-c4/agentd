/**
 * Agentd Plan Viewer Application
 * Uses REST API for backend communication
 */

// Global state
const state = {
    changeId: 'unknown',
    files: [],
    currentFile: null,
    annotations: [],
    headings: [],
    pendingSave: null,
    pendingResolve: null
};

// ============================================================================
// API Communication (using fetch)
// ============================================================================

async function apiGet(endpoint) {
    try {
        const response = await fetch(endpoint);
        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.error || `HTTP ${response.status}`);
        }
        return await response.json();
    } catch (e) {
        console.error(`API GET ${endpoint} failed:`, e);
        throw e;
    }
}

async function apiPost(endpoint, data = {}) {
    try {
        const response = await fetch(endpoint, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data)
        });
        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.error || `HTTP ${response.status}`);
        }
        return await response.json();
    } catch (e) {
        console.error(`API POST ${endpoint} failed:`, e);
        throw e;
    }
}

// ============================================================================
// Toast Notifications
// ============================================================================

function showToast(message, type = 'info') {
    const container = document.getElementById('toast-container');
    const toast = document.createElement('div');
    toast.className = `toast ${type}`;
    toast.textContent = message;
    container.appendChild(toast);

    setTimeout(() => {
        toast.remove();
    }, 5000);
}

// ============================================================================
// File Navigation
// ============================================================================

function initFileNav() {
    const nav = document.getElementById('file-nav');
    const changeIdEl = document.getElementById('change-id');

    changeIdEl.textContent = state.changeId;

    const icons = {
        'proposal.md': '📝',
        'CHALLENGE.md': '🔍',
        'STATE.yaml': '📊',
        'tasks.md': '✅'
    };

    nav.innerHTML = state.files.map(file => `
        <div class="file-item ${file.exists ? '' : 'missing'}" data-file="${file.name}">
            <span class="file-icon">${icons[file.name] || (file.name.startsWith('specs/') ? '📋' : '📄')}</span>
            <span>${file.name}</span>
        </div>
    `).join('');

    // Add click handlers
    nav.querySelectorAll('.file-item').forEach(item => {
        item.addEventListener('click', () => {
            const filename = item.dataset.file;
            loadFile(filename);
        });
    });

    // Load first available file
    const firstFile = state.files.find(f => f.exists);
    if (firstFile) {
        loadFile(firstFile.name);
    }
}

async function loadFile(filename) {
    const nav = document.getElementById('file-nav');
    const header = document.getElementById('current-file');
    const body = document.getElementById('content-body');

    // Update active state
    nav.querySelectorAll('.file-item').forEach(item => {
        item.classList.toggle('active', item.dataset.file === filename);
    });

    header.textContent = filename;
    state.currentFile = filename;

    // Show loading state
    body.innerHTML = '<div class="placeholder"><p>Loading...</p></div>';

    try {
        const response = await apiGet(`/api/files/${filename}`);
        updateContent(response.content, response.annotations || []);
    } catch (e) {
        body.innerHTML = `<div class="placeholder"><p>Error loading file: ${e.message}</p></div>`;
        showToast(`Failed to load ${filename}: ${e.message}`, 'error');
    }
}

// ============================================================================
// Content Rendering
// ============================================================================

function updateContent(html, annotations) {
    const body = document.getElementById('content-body');
    body.innerHTML = html;

    state.annotations = annotations || [];
    state.headings = extractHeadings();

    // Initialize mermaid diagrams
    initMermaid();

    // Initialize syntax highlighting
    initHighlight();

    // Update annotations display
    renderAnnotations();

    // Add click handlers to headings
    initHeadingClickHandlers();
}

function extractHeadings() {
    const body = document.getElementById('content-body');
    const headings = [];

    body.querySelectorAll('h1, h2, h3, h4, h5, h6').forEach(h => {
        if (h.id) {
            headings.push({
                id: h.id,
                text: h.textContent,
                level: parseInt(h.tagName[1])
            });
        }
    });

    return headings;
}

function initMermaid() {
    if (typeof mermaid !== 'undefined') {
        mermaid.initialize({
            startOnLoad: false,
            theme: 'dark',
            securityLevel: 'strict'
        });

        document.querySelectorAll('code.language-mermaid').forEach(block => {
            const parent = block.parentElement;
            const div = document.createElement('div');
            div.className = 'mermaid';
            div.textContent = block.textContent;
            parent.replaceWith(div);
        });

        mermaid.run();
    }
}

function initHighlight() {
    if (typeof hljs !== 'undefined') {
        document.querySelectorAll('pre code:not(.language-mermaid)').forEach(block => {
            hljs.highlightElement(block);
        });
    }
}

// ============================================================================
// Annotations
// ============================================================================

function renderAnnotations() {
    const list = document.getElementById('annotations-list');

    // Update global comment count
    updateCommentCount();

    const fileAnnotations = state.annotations.filter(a => a.file === state.currentFile);

    if (fileAnnotations.length === 0) {
        list.innerHTML = '<div class="no-annotations">No annotations yet</div>';
        return;
    }

    list.innerHTML = fileAnnotations.map(a => `
        <div class="annotation-card ${a.resolved ? 'resolved' : ''}" data-id="${escapeHtml(a.id)}">
            <div class="annotation-section">#${escapeHtml(a.section_id)}</div>
            <div class="annotation-content">${escapeHtml(a.content)}</div>
            <div class="annotation-meta">
                <span>${escapeHtml(a.author)} • ${formatDate(a.created_at)}</span>
                <div class="annotation-actions">
                    ${!a.resolved ? `<button onclick="resolveAnnotation('${escapeHtml(a.id)}')" title="Resolve">✓</button>` : ''}
                </div>
            </div>
        </div>
    `).join('');
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function formatDate(isoString) {
    try {
        const date = new Date(isoString);
        return date.toLocaleDateString();
    } catch {
        return isoString;
    }
}

async function resolveAnnotation(id) {
    // Optimistic update
    const annotation = state.annotations.find(a => a.id === id);
    if (annotation) {
        annotation.resolved = true;
        renderAnnotations();
    }

    try {
        await apiPost(`/api/annotations/${encodeURIComponent(id)}/resolve`);
        showToast('Annotation resolved', 'success');
    } catch (e) {
        // Rollback on failure
        if (annotation) {
            annotation.resolved = false;
            renderAnnotations();
        }
        showToast(`Failed to resolve: ${e.message}`, 'error');
    }
}

// Make resolveAnnotation globally accessible for onclick handlers
window.resolveAnnotation = resolveAnnotation;

// ============================================================================
// Heading Click Handlers
// ============================================================================

function initHeadingClickHandlers() {
    const body = document.getElementById('content-body');

    body.querySelectorAll('h1[id], h2[id], h3[id], h4[id], h5[id], h6[id]').forEach(h => {
        // Skip if button already added
        if (h.parentElement && h.parentElement.classList.contains('section-wrapper')) {
            return;
        }

        // Wrap heading in a relative container
        const wrapper = document.createElement('div');
        wrapper.className = 'section-wrapper';
        h.parentNode.insertBefore(wrapper, h);
        wrapper.appendChild(h);

        // Create the visible comment button
        const btn = document.createElement('button');
        btn.className = 'section-comment-btn';
        btn.title = 'Add comment to this section';
        btn.innerHTML = '<span class="icon">💬</span><span>Comment</span>';
        btn.addEventListener('click', (e) => {
            e.stopPropagation();
            openAnnotationModal(h.id, h.textContent);
        });
        wrapper.appendChild(btn);

        // Also keep heading clickable for convenience
        h.style.cursor = 'pointer';
        h.addEventListener('click', () => {
            openAnnotationModal(h.id, h.textContent);
        });
    });
}

// ============================================================================
// Annotation Modal
// ============================================================================

function openAnnotationModal(sectionId, sectionText) {
    const modal = document.getElementById('annotation-modal');
    const sectionSelect = document.getElementById('annotation-section');
    const contentInput = document.getElementById('annotation-content');

    // Populate section dropdown
    sectionSelect.innerHTML = state.headings.map(h => `
        <option value="${h.id}" ${h.id === sectionId ? 'selected' : ''}>
            ${'  '.repeat(h.level - 1)}${h.text}
        </option>
    `).join('');

    contentInput.value = '';
    modal.style.display = 'flex';
    contentInput.focus();
}

function closeAnnotationModal() {
    const modal = document.getElementById('annotation-modal');
    modal.style.display = 'none';
}

async function saveAnnotation() {
    const sectionId = document.getElementById('annotation-section').value;
    const content = document.getElementById('annotation-content').value.trim();

    if (!content) {
        showToast('Please enter a comment', 'warning');
        return;
    }

    const saveBtn = document.getElementById('btn-save');
    saveBtn.textContent = 'Saving...';
    saveBtn.disabled = true;

    try {
        const annotation = await apiPost('/api/annotations', {
            file: state.currentFile,
            section_id: sectionId,
            content
        });

        state.annotations.push(annotation);
        renderAnnotations();
        closeAnnotationModal();
        showToast('Annotation saved', 'success');
    } catch (e) {
        showToast(`Failed to save: ${e.message}`, 'error');
    } finally {
        saveBtn.textContent = 'Save';
        saveBtn.disabled = false;
    }
}

function initModal() {
    const modal = document.getElementById('annotation-modal');
    const closeBtn = document.getElementById('modal-close');
    const cancelBtn = document.getElementById('btn-cancel');
    const saveBtn = document.getElementById('btn-save');
    const addBtn = document.getElementById('btn-add-annotation');

    closeBtn.addEventListener('click', closeAnnotationModal);
    cancelBtn.addEventListener('click', closeAnnotationModal);
    saveBtn.addEventListener('click', saveAnnotation);
    addBtn.addEventListener('click', () => {
        if (state.headings.length > 0) {
            openAnnotationModal(state.headings[0].id, state.headings[0].text);
        } else {
            showToast('No sections available for annotation', 'warning');
        }
    });

    // Close on backdrop click
    modal.addEventListener('click', (e) => {
        if (e.target === modal) {
            closeAnnotationModal();
        }
    });

    // Close on Escape key
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape' && modal.style.display === 'flex') {
            closeAnnotationModal();
        }
    });
}

// ============================================================================
// Review Actions (Global - for entire proposal)
// ============================================================================

function updateCommentCount() {
    const countEl = document.getElementById('comment-count');
    const unresolvedCount = state.annotations.filter(a => !a.resolved).length;
    countEl.textContent = unresolvedCount === 0
        ? 'No comments'
        : `${unresolvedCount} comment${unresolvedCount > 1 ? 's' : ''}`;
}

function initReviewActions() {
    const approveBtn = document.getElementById('btn-approve');
    const requestChangesBtn = document.getElementById('btn-request-changes');

    approveBtn.addEventListener('click', async () => {
        if (!confirm('Approve this proposal and mark as complete?')) return;

        approveBtn.disabled = true;
        requestChangesBtn.disabled = true;
        approveBtn.innerHTML = '<span class="btn-icon">⏳</span> Approving...';

        try {
            await apiPost('/api/review/approve');
            showToast('Proposal approved!', 'success');
            // Close server and browser
            setTimeout(async () => {
                await apiPost('/api/close').catch(() => {});
                window.close();
            }, 1000);
        } catch (e) {
            showToast(`Failed to approve: ${e.message}`, 'error');
            approveBtn.disabled = false;
            requestChangesBtn.disabled = false;
            approveBtn.innerHTML = `
                <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="20 6 9 17 4 12"></polyline>
                </svg>
                Approve
            `;
        }
    });

    requestChangesBtn.addEventListener('click', async () => {
        const unresolvedCount = state.annotations.filter(a => !a.resolved).length;
        if (unresolvedCount === 0) {
            showToast('Add at least one comment before requesting changes', 'warning');
            return;
        }

        if (!confirm(`Request changes with ${unresolvedCount} comment(s)?`)) return;

        approveBtn.disabled = true;
        requestChangesBtn.disabled = true;
        requestChangesBtn.innerHTML = '<span class="btn-icon">⏳</span> Submitting...';

        try {
            await apiPost('/api/review/request-changes');
            showToast('Changes requested!', 'success');
            // Close server and browser
            setTimeout(async () => {
                await apiPost('/api/close').catch(() => {});
                window.close();
            }, 1000);
        } catch (e) {
            showToast(`Failed: ${e.message}`, 'error');
            approveBtn.disabled = false;
            requestChangesBtn.disabled = false;
            requestChangesBtn.innerHTML = `
                <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                    <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                </svg>
                Request Changes
            `;
        }
    });
}

// ============================================================================
// Initialization
// ============================================================================

async function init() {
    try {
        // Fetch initial info from server
        const info = await apiGet('/api/info');
        state.changeId = info.change_id;
        state.files = info.files;

        initFileNav();
        initModal();
        initReviewActions();
    } catch (e) {
        console.error('Failed to initialize:', e);
        showToast(`Failed to connect to server: ${e.message}`, 'error');
    }
}

document.addEventListener('DOMContentLoaded', init);

// Expose for debugging
window.agentd = {
    state,
    loadFile,
    showToast
};
