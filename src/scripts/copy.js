var timeout;

// Modern copy function using Clipboard API with fallback
async function copy(text) {
	const unescaped = unEscapeHtml(text);

	try {
		// Try modern Clipboard API first
		if (navigator.clipboard && window.isSecureContext) {
			await navigator.clipboard.writeText(unescaped);
			showToast('✓ Copied to clipboard!', 'success');
			animateCopyButton(event?.target);
		} else {
			// Fallback for older browsers or non-secure contexts
			copyFallback(unescaped);
			showToast('✓ Copied to clipboard!', 'success');
			animateCopyButton(event?.target);
		}
	} catch (err) {
		console.error('Copy failed:', err);
		// Try fallback method if modern API fails
		try {
			copyFallback(unescaped);
			showToast('✓ Copied to clipboard!', 'success');
			animateCopyButton(event?.target);
		} catch (fallbackErr) {
			console.error('Fallback copy failed:', fallbackErr);
			showToast('✗ Failed to copy', 'error');
		}
	}
}

// Fallback copy method for older browsers
function copyFallback(text) {
	var inp = document.createElement('textarea');
	inp.style.position = 'fixed';
	inp.style.opacity = '0';
	document.body.appendChild(inp);
	inp.value = text;
	inp.select();
	document.execCommand('copy', false);
	inp.remove();
}

// Modern toast notification instead of tooltip
function showToast(message, type = 'success') {
	// Remove existing toast if any
	const existingToast = document.querySelector('.toast-notification');
	if (existingToast) {
		existingToast.remove();
	}

	// Create toast element
	const toast = document.createElement('div');
	toast.className = `toast-notification toast-${type}`;
	toast.textContent = message;

	// Add to body
	document.body.appendChild(toast);

	// Trigger animation
	setTimeout(() => {
		toast.classList.add('show');
	}, 10);

	// Remove after delay
	clearTimeout(timeout);
	timeout = setTimeout(() => {
		toast.classList.remove('show');
		setTimeout(() => {
			toast.remove();
		}, 300);
	}, 2000);
}

// Animate copy button to provide visual feedback
function animateCopyButton(button) {
	if (!button) return;

	// Find the button element if event target is the icon inside
	const copyButton = button.closest('.copy-button') || button;

	// Add animation class
	copyButton.classList.add('copy-success');

	// Remove animation class after animation completes
	setTimeout(() => {
		copyButton.classList.remove('copy-success');
	}, 600);
}

// Deprecated functions kept for backward compatibility
function show() {
	// Deprecated - now uses toast notifications
}

function hide() {
	// Deprecated - now uses toast notifications
}

function unEscapeHtml(unsafe) {
	return unsafe
		.replace(/&amp;/g, "&")
		.replace(/&lt;/g, "<")
		.replace(/&gt;/g, ">")
		.replace(/&quot;/g, "\"")
		.replace(/&#039;/g, "'")
		.replace(/&#47;/g, "/")
		.replace(/\\/g, "\\\\");
}