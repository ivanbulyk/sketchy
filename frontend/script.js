document.addEventListener('DOMContentLoaded', () => {
    const API_BASE_URL = '/api/v1';

    // State variables
    let uploadedFiles = [];
    let selectedFileId = null;
    let analysisId = null;
    let regeneratedImageId = null;
    let lastImprovedImageId = null;

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
    const errorContainer = document.getElementById('error-container');
    const errorMessage = document.getElementById('error-message');

    // --- 1. UPLOAD --- //
    uploadButton.addEventListener('click', () => fileInput.click());
    fileInput.addEventListener('change', handleFileUpload);

    async function handleFileUpload(event) {
        const files = event.target.files;
        if (files.length === 0) return;

        const formData = new FormData();
        for (const file of files) {
            formData.append('images', file);
        }

        setLoadingState(uploadStatus, 'Uploading...', uploadButton);
        try {
            const response = await fetch(`${API_BASE_URL}/upload`, {
                method: 'POST',
                body: formData,
            });

            const result = await response.json();
            if (!response.ok) {
                throw new Error(result.message || 'Upload failed');
            }

            uploadStatus.textContent = `${result.count} file(s) uploaded successfully. Session ID: ${result.session_id}`;
            await fetchUploadedFiles(result.uploaded_images, files);
            analysisSection.style.display = 'block';
        } catch (error) {
            showError(error);
            uploadStatus.textContent = 'Upload failed.';
        } finally {
            clearLoadingState(uploadButton);
        }
    }

    async function fetchUploadedFiles(imageIds, fileList) {
        uploadedFiles = [];
        for (let i = 0; i < imageIds.length; i++) {
            uploadedFiles.push({ 
                id: imageIds[i], 
                file: fileList[i] 
            });
        }
        renderFileList();
    }

    function renderFileList() {
        fileList.innerHTML = '';
        uploadedFiles.forEach(fileData => {
            const li = document.createElement('li');
            li.dataset.id = fileData.id;
            li.innerHTML = `<img src="${URL.createObjectURL(fileData.file)}" alt="preview"> ${fileData.file.name}`;
            li.addEventListener('click', () => selectFileForAnalysis(fileData.id));
            fileList.appendChild(li);
        });
    }

    // --- 2. ANALYSIS --- //
    function selectFileForAnalysis(fileId) {
        selectedFileId = fileId;
        const fileData = uploadedFiles.find(f => f.id === fileId);

        // Highlight selected item
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

    analyzeButton.addEventListener('click', handleAnalysis);

    async function handleAnalysis() {
        if (!selectedFileId) return;

        const provider = document.querySelector('input[name="llm-provider"]:checked').value;
        setLoadingState(analysisStatus, 'Analyzing image...', analyzeButton);
        try {
            const response = await fetch(`${API_BASE_URL}/analyze/${selectedFileId}?provider=${provider}`, {
                method: 'POST',
            });
            const result = await response.json();
            if (!response.ok) {
                throw new Error(result.message || 'Analysis failed');
            }

            analysisId = result.id;
            promptInput.value = result.prompt_description;
            analysisStatus.textContent = `Analysis complete. Analysis ID: ${analysisId}`;
            regenerationSection.style.display = 'block';
            regenerateButton.disabled = false;
        } catch (error) {
            showError(error);
            analysisStatus.textContent = 'Analysis failed.';
        } finally {
            clearLoadingState(analyzeButton);
        }
    }

    // --- 3. REGENERATION --- //
    regenerateButton.addEventListener('click', handleRegeneration);

    async function handleRegeneration() {
        if (!analysisId) return;

        const prompt = promptInput.value;
        if (!prompt) {
            showError({ message: 'Prompt cannot be empty.' });
            return;
        }

        setLoadingState(regenerationStatus, 'Regenerating image...', regenerateButton);
        try {
            const response = await fetch(`${API_BASE_URL}/regenerate/${analysisId}`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ prompt: prompt, provider: 'stabilityai' }), // Or make this selectable
            });

            const result = await response.json();
            if (!response.ok) {
                throw new Error(result.message || 'Regeneration failed');
            }

            regeneratedImageId = result.id;
            lastImprovedImageId = null; // Reset improvement chain
            regeneratedImage.src = `data:image/png;base64,${result.data}`;
            regenerationStatus.textContent = `Regeneration complete. Image ID: ${regeneratedImageId}`;
            improvementSection.style.display = 'block';
            improveButton.disabled = false;
        } catch (error) {
            showError(error);
            regenerationStatus.textContent = 'Regeneration failed.';
        } finally {
            clearLoadingState(regenerateButton);
        }
    }

    // --- 4. IMPROVEMENT --- //
    improveButton.addEventListener('click', handleImprovement);

    async function handleImprovement() {
        const prompt = improvementPrompt.value;
        if (!prompt) {
            showError({ message: 'Improvement prompt cannot be empty.' });
            return;
        }

        let url;
        let bodyId;

        if (lastImprovedImageId) {
            // Continue the chain
            url = `${API_BASE_URL}/improve/from_improved/${lastImprovedImageId}`;
        } else {
            // Start a new chain
            url = `${API_BASE_URL}/improve/from_original/${regeneratedImageId}`;
        }

        setLoadingState(improvementStatus, 'Improving image...', improveButton);
        try {
            const response = await fetch(url, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ prompt }),
            });

            const result = await response.json();
            if (!response.ok) {
                throw new Error(result.message || 'Improvement failed');
            }

            lastImprovedImageId = result.id;
            regeneratedImage.src = `data:image/png;base64,${result.data}`;
            improvementStatus.textContent = `Improvement complete. New Image ID: ${lastImprovedImageId}`;
            improvementPrompt.value = ''; // Clear prompt after use
        } catch (error) {
            showError(error);
            improvementStatus.textContent = 'Improvement failed.';
        } finally {
            clearLoadingState(improveButton);
        }
    }

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
});
