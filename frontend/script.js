document.addEventListener('DOMContentLoaded', () => {
    const API_BASE_URL = '/api/v1';

    // State object
    let state = {
        uploadedFilesInfo: [],
        selectedFileId: null,
        analysisId: null,
        analysisPrompt: '',
        regeneratedImageId: null,
        regeneratedImageData: null,
        lastImprovedImageId: null,
    };

    // DOM Elements
    const uploadButton = document.getElementById('upload-button');
    const fileInput = document.getElementById('file-input');
    const uploadStatus = document.getElementById('upload-status');
    const analysisSection = document.getElementById('analysis-section');
    const fileList = document.getElementById('file-list');
    const imagePreview = document.getElementById('image-preview');
    const analyzeButton = document.getElementById('analyze-button');
    const analysisStatus = document.getElementById('analysis-status');
    const regenerationSection = document.getElementById('regeneration-section');
    const promptInput = document.getElementById('prompt-input');
    const regenerateButton = document.getElementById('regenerate-button');
    const regenerationStatus = document.getElementById('regeneration-status');
    const improvementSection = document.getElementById('improvement-section');
    const regeneratedImage = document.getElementById('regenerated-image');
    const improvementPrompt = document.getElementById('improvement-prompt');
    const improveButton = document.getElementById('improve-button');
    const improvementStatus = document.getElementById('improvement-status');
    const saveImageButton = document.getElementById('save-image-button');
    const enlargeImageButton = document.getElementById('enlarge-image-button');
    const errorContainer = document.getElementById('error-container');
    const errorMessage = document.getElementById('error-message');
    const imageModal = document.getElementById('image-modal');
    const enlargedImage = document.getElementById('enlarged-image');
    const closeModal = document.querySelector('.close-modal');

    // --- STATE MANAGEMENT --- //
    function saveState() {
        const stateToSave = {
            ...state,
            uploadedFilesInfo: state.uploadedFilesInfo.map(f => ({ id: f.id, name: f.file.name, type: f.file.type }))
        };
        localStorage.setItem('sketchyState', JSON.stringify(stateToSave));
    }

    function loadState() {
        const savedState = localStorage.getItem('sketchyState');
        if (savedState) {
            const parsedState = JSON.parse(savedState);
            if (parsedState.uploadedFilesInfo && parsedState.uploadedFilesInfo.length > 0) {
                alert("To restore your session, please re-select the same files you uploaded previously.");
                fileInput.click();
            }
            state = { ...state, ...parsedState };
        }
    }

    function restoreUIFromState() {
        if (state.uploadedFilesInfo.length > 0) {
            analysisSection.style.display = 'block';
            renderFileList();
        }
        if (state.selectedFileId) {
            selectFileForAnalysis(state.selectedFileId, true);
        }
        if (state.analysisId) {
            regenerationSection.style.display = 'block';
            promptInput.value = state.analysisPrompt;
            regenerateButton.disabled = false;
            analysisStatus.textContent = `Analysis restored. ID: ${state.analysisId}`;
        }
        if (state.regeneratedImageId) {
            improvementSection.style.display = 'block';
            regeneratedImage.src = state.regeneratedImageData;
            regeneratedImage.style.display = 'block';
            improveButton.disabled = false;
            saveImageButton.disabled = false;
            enlargeImageButton.disabled = false;
            regenerationStatus.textContent = `Image restored. ID: ${state.regeneratedImageId}`;
        }
    }

    // --- 1. UPLOAD --- //
    uploadButton.addEventListener('click', () => fileInput.click());
    fileInput.addEventListener('change', handleFileUpload);

    async function handleFileUpload(event) {
        const files = event.target.files;
        if (files.length === 0) return;

        const isRestoring = state.uploadedFilesInfo.length > 0 && !state.uploadedFilesInfo[0].file;
        if (isRestoring) {
            const restoredFiles = [];
            state.uploadedFilesInfo.forEach(info => {
                const foundFile = Array.from(files).find(f => f.name === info.name && f.type === info.type);
                if (foundFile) {
                    restoredFiles.push({ id: info.id, file: foundFile });
                }
            });
            state.uploadedFilesInfo = restoredFiles;
            restoreUIFromState();
            return;
        }

        const formData = new FormData();
        for (const file of files) {
            formData.append('images', file);
        }

        setLoadingState(uploadStatus, 'Uploading...', uploadButton);
        try {
            const response = await fetch(`${API_BASE_URL}/upload`, { method: 'POST', body: formData });
            const result = await response.json();
            if (!response.ok) throw new Error(result.message || 'Upload failed');

            uploadStatus.textContent = `${result.count} file(s) uploaded. Session ID: ${result.session_id}`;
            state.uploadedFilesInfo = [];
            for (let i = 0; i < result.uploaded_images.length; i++) {
                state.uploadedFilesInfo.push({ id: result.uploaded_images[i], file: files[i] });
            }
            saveState();
            analysisSection.style.display = 'block';
            renderFileList();
        } catch (error) {
            showError(error);
            uploadStatus.textContent = 'Upload failed.';
        } finally {
            clearLoadingState(uploadButton);
        }
    }

    function renderFileList() {
        fileList.innerHTML = '';
        state.uploadedFilesInfo.forEach(fileData => {
            const li = document.createElement('li');
            li.dataset.id = fileData.id;
            li.innerHTML = `<img src="${URL.createObjectURL(fileData.file)}" alt="preview"> ${fileData.file.name}`;
            li.addEventListener('click', () => selectFileForAnalysis(fileData.id));
            fileList.appendChild(li);
        });
    }

    // --- 2. ANALYSIS --- //
    function selectFileForAnalysis(fileId, isRestoring = false) {
        state.selectedFileId = fileId;
        if (!isRestoring) saveState();

        const fileData = state.uploadedFilesInfo.find(f => f.id === fileId);
        document.querySelectorAll('#file-list li').forEach(li => {
            li.classList.toggle('selected', li.dataset.id === fileId);
        });

        if (fileData) {
            imagePreview.src = URL.createObjectURL(fileData.file);
            imagePreview.style.display = 'block';
            analyzeButton.disabled = false;
        } else {
            imagePreview.style.display = 'none';
            analyzeButton.disabled = true;
        }
    }

    analyzeButton.addEventListener('click', async () => {
        if (!state.selectedFileId) return;
        const provider = document.querySelector('input[name="llm-provider"]:checked').value;
        setLoadingState(analysisStatus, 'Analyzing image...', analyzeButton);
        try {
            const response = await fetch(`${API_BASE_URL}/analyze/${state.selectedFileId}?provider=${provider}`, { method: 'POST' });
            const result = await response.json();
            if (!response.ok) throw new Error(result.message || 'Analysis failed');

            state.analysisId = result.id;
            state.analysisPrompt = result.prompt_description;
            saveState();

            promptInput.value = state.analysisPrompt;
            analysisStatus.textContent = `Analysis complete. ID: ${state.analysisId}`;
            regenerationSection.style.display = 'block';
            regenerateButton.disabled = false;
        } catch (error) {
            showError(error);
            analysisStatus.textContent = 'Analysis failed.';
        } finally {
            clearLoadingState(analyzeButton);
        }
    });

    // --- 3. REGENERATION --- //
    regenerateButton.addEventListener('click', async () => {
        if (!state.analysisId) return;
        const prompt = promptInput.value;
        if (!prompt) {
            showError({ message: 'Prompt cannot be empty.' });
            return;
        }
        state.analysisPrompt = prompt;

        setLoadingState(regenerationStatus, 'Regenerating image...', regenerateButton);
        try {
            const response = await fetch(`${API_BASE_URL}/regenerate/${state.analysisId}`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ prompt: state.analysisPrompt, provider: 'stabilityai' }),
            });
            const result = await response.json();
            if (!response.ok) throw new Error(result.message || 'Regeneration failed');

            state.regeneratedImageId = result.id;
            state.regeneratedImageData = `data:image/png;base64,${result.data}`;
            state.lastImprovedImageId = null;
            saveState();

            regeneratedImage.src = state.regeneratedImageData;
            regeneratedImage.style.display = 'block';
            regenerationStatus.textContent = `Regeneration complete. ID: ${state.regeneratedImageId}`;
            improvementSection.style.display = 'block';
            improveButton.disabled = false;
            saveImageButton.disabled = false;
            enlargeImageButton.disabled = false;
        } catch (error) {
            showError(error);
            regenerationStatus.textContent = 'Regeneration failed.';
        } finally {
            clearLoadingState(regenerateButton);
        }
    });

    // --- 4. IMPROVEMENT --- //
    improveButton.addEventListener('click', async () => {
        const prompt = improvementPrompt.value;
        if (!prompt) {
            showError({ message: 'Improvement prompt cannot be empty.' });
            return;
        }

        const url = state.lastImprovedImageId
            ? `${API_BASE_URL}/improve/from_improved/${state.lastImprovedImageId}`
            : `${API_BASE_URL}/improve/from_original/${state.regeneratedImageId}`;

        setLoadingState(improvementStatus, 'Improving image...', improveButton);
        try {
            const response = await fetch(url, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ prompt }),
            });
            const result = await response.json();
            if (!response.ok) throw new Error(result.message || 'Improvement failed');

            state.lastImprovedImageId = result.id;
            state.regeneratedImageData = `data:image/png;base64,${result.data}`;
            saveState();

            regeneratedImage.src = state.regeneratedImageData;
            improvementStatus.textContent = `Improvement complete. New ID: ${state.lastImprovedImageId}`;
            improvementPrompt.value = '';
        } catch (error) {
            showError(error);
            improvementStatus.textContent = 'Improvement failed.';
        } finally {
            clearLoadingState(improveButton);
        }
    });

    // --- IMAGE ACTIONS & MODAL --- //
    saveImageButton.addEventListener('click', () => {
        const link = document.createElement('a');
        link.href = state.regeneratedImageData;
        link.download = `sketchy-image-${Date.now()}.png`;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
    });

    enlargeImageButton.addEventListener('click', () => {
        enlargedImage.src = state.regeneratedImageData;
        imageModal.style.display = 'block';
    });

    closeModal.addEventListener('click', () => imageModal.style.display = 'none');
    window.addEventListener('click', (event) => {
        if (event.target == imageModal) {
            imageModal.style.display = 'none';
        }
    });

    // --- UTILITY FUNCTIONS --- //
    function setLoadingState(statusElement, text, button) {
        clearError();
        statusElement.textContent = text;
        if (button) button.disabled = true;
    }

    function clearLoadingState(button) {
        if (button) button.disabled = false;
    }

    function showError(error) {
        errorMessage.textContent = error.message || JSON.stringify(error, null, 2);
        errorContainer.style.display = 'block';
    }

    function clearError() {
        errorContainer.style.display = 'none';
        errorMessage.textContent = '';
    }

    // --- INITIALIZATION --- //
    loadState();
});
