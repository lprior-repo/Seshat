#[cfg(target_arch = "wasm32")]
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
pub fn use_dev_shell_cleanup() {
    use_effect(move || {
        let _eval = document::eval(
            r#"
                (() => {
                    if (typeof window.__seshatDevShellCleanup === "function") {
                        window.__seshatDevShellCleanup();
                    }
                    const staleToastText = "Your app is being rebuilt. A non-hot-reloadable change occurred and we must rebuild.";
                    const normalize = (value) => (value || "").replace(/\s+/g, " ").trim();
                    const isExactStaleDioxusToast = (node) => {
                        if (!(node instanceof HTMLElement)) {
                            return false;
                        }
                        const isDioxusToast = node.id === "__dx-toast-inner" || node.classList.contains("dx-toast-inner");
                        return isDioxusToast && normalize(node.textContent) === staleToastText;
                    };
                    const cleanup = () => {
                        if (!document.querySelector('[data-testid="canvas-root"]')) {
                            return;
                        }
                        document.body.dataset.seshatHydrated = "true";
                        document.querySelectorAll('#__dx-toast-inner, .dx-toast-inner').forEach((node) => {
                            if (isExactStaleDioxusToast(node)) {
                                node.remove();
                            }
                        });
                    };
                    cleanup();
                    requestAnimationFrame(cleanup);
                    setTimeout(cleanup, 250);
                    const observer = new MutationObserver(cleanup);
                    observer.observe(document.body, { childList: true, subtree: true });
                    const timeout = setTimeout(() => {
                        observer.disconnect();
                        if (window.__seshatDevShellCleanup === disconnect) {
                            delete window.__seshatDevShellCleanup;
                        }
                    }, 10000);
                    const disconnect = () => {
                        clearTimeout(timeout);
                        observer.disconnect();
                    };
                    window.__seshatDevShellCleanup = disconnect;
                })();
            "#,
        );
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn use_dev_shell_cleanup() {}
