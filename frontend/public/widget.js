/**
 * FidduPay Checkout Widget
 * Integrates crypto payments beautifully into any website framework.
 */
(function(window) {
    if (window.FidduPay) return; // Already injected

    const FidduPay = {
        iframeUrl: '',
        
        init: function() {
            // Auto detect the origin of this script to know where to load the UI iframe from
            const scripts = document.getElementsByTagName('script');
            for (let i = 0; i < scripts.length; i++) {
                if (scripts[i].src.includes('widget.js')) {
                    const url = new URL(scripts[i].src);
                    this.iframeUrl = url.origin;
                    break;
                }
            }
            if (!this.iframeUrl) this.iframeUrl = 'https://pay.fiddupay.com';
            
            this.injectStyles();
        },

        injectStyles: function() {
            if (document.getElementById('fiddupay-widget-styles')) return;
            
            const style = document.createElement('style');
            style.id = 'fiddupay-widget-styles';
            style.innerHTML = `
                .fiddupay-backdrop {
                    position: fixed;
                    top: 0; left: 0; width: 100vw; height: 100vh;
                    background: rgba(15, 23, 42, 0.6);
                    backdrop-filter: blur(8px);
                    -webkit-backdrop-filter: blur(8px);
                    z-index: 999999;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    opacity: 0;
                    transition: opacity 0.3s ease;
                }
                .fiddupay-backdrop.active {
                    opacity: 1;
                }
                .fiddupay-modal-container {
                    width: 100%;
                    max-width: 480px;
                    height: 90vh;
                    max-height: 800px;
                    background: transparent;
                    border-radius: 24px;
                    overflow: hidden;
                    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
                    transform: translateY(20px) scale(0.95);
                    transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1);
                    position: relative;
                }
                .fiddupay-backdrop.active .fiddupay-modal-container {
                    transform: translateY(0) scale(1);
                }
                .fiddupay-iframe {
                    width: 100%;
                    height: 100%;
                    border: none;
                    background: transparent;
                }
                .fiddupay-close-btn {
                    position: absolute;
                    top: 16px;
                    right: 16px;
                    background: rgba(255, 255, 255, 0.1);
                    border: 1px solid rgba(255, 255, 255, 0.2);
                    color: white;
                    width: 36px;
                    height: 36px;
                    border-radius: 50%;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    cursor: pointer;
                    z-index: 10;
                    backdrop-filter: blur(4px);
                    font-size: 20px;
                    line-height: 1;
                    transition: all 0.2s ease;
                }
                .fiddupay-close-btn:hover {
                    background: rgba(255, 255, 255, 0.2);
                    transform: scale(1.05);
                }
                @media (max-width: 500px) {
                    .fiddupay-modal-container {
                        height: 100vh;
                        max-height: 100vh;
                        max-width: 100vw;
                        border-radius: 0;
                    }
                    .fiddupay-close-btn {
                        top: 12px;
                        right: 12px;
                        background: rgba(0, 0, 0, 0.5);
                    }
                }
            `;
            document.head.appendChild(style);
        },

        open: async function(config) {
            // Configuration payload
            const theme = config.theme || 'dark';
            let paymentLink = config.paymentLink || '';

            if (!paymentLink && !config.publicKey) {
                console.error('FidduPay Widget Error: You must provide either a paymentLink or a publicKey mapping.');
                return;
            }

            // Create Backdrop
            const backdrop = document.createElement('div');
            backdrop.className = 'fiddupay-backdrop';
            backdrop.id = 'fiddupay-overlay';
            
            // Create Modal Container
            const modal = document.createElement('div');
            modal.className = 'fiddupay-modal-container';

            // Close Button
            const closeBtn = document.createElement('button');
            closeBtn.className = 'fiddupay-close-btn';
            closeBtn.innerHTML = '&times;';
            closeBtn.onclick = () => this.close();

            // Create IFrame loading the generic widget route
            const iframe = document.createElement('iframe');
            iframe.className = 'fiddupay-iframe';
            iframe.allow = 'clipboard-read; clipboard-write';

            // Loading state
            iframe.srcdoc = `
                <html>
                <head>
                    <style>
                        body { background: transparent; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; color: white; font-family: sans-serif; }
                        .loader { border: 4px solid rgba(255,255,255,0.1); border-left-color: #3b82f6; border-radius: 50%; width: 40px; height: 40px; animation: spin 1s linear infinite; }
                        @keyframes spin { 0% { transform: rotate(0deg); } 100% { transform: rotate(360deg); } }
                    </style>
                </head>
                <body><div class="loader"></div></body>
                </html>
            `;

            // Assemble and Show early so it feels fast
            modal.appendChild(closeBtn);
            modal.appendChild(iframe);
            backdrop.appendChild(modal);
            document.body.appendChild(backdrop);
            document.body.style.overflow = 'hidden';

            // Animate in
            setTimeout(() => { backdrop.classList.add('active'); }, 10);

            // Setup cross-origin message listener
            this._setupMessageListener(config);

            // Phase 2: If we are using pure Public No-Code Key, generate dynamic invoice
            if (!paymentLink && config.publicKey) {
                try {
                    const apiUrl = config.apiUrl || (this.iframeUrl + '/api/v1');
                    const res = await fetch(`${apiUrl}/public/payments/create`, {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({
                            publishable_key: config.publicKey,
                            amount: config.amount ? String(config.amount) : null,
                            amount_usd: config.amountUsd ? String(config.amountUsd) : null,
                            description: config.description || 'Zero-Code Widget Checkout'
                        })
                    });

                    if (!res.ok) throw new Error('API Rejection: ' + res.statusText);
                    const data = await res.json();
                    if (!data.payment_id) throw new Error('Invalid response structure');
                    
                    paymentLink = data.payment_id;
                } catch (e) {
                    console.error('FidduPay Public Key Flow Error:', e);
                    iframe.srcdoc = '<html><body style="color:white;text-align:center;font-family:sans-serif;margin-top:50px;">Failed to initialize secure checkout. Please check the console.</body></html>';
                    return;
                }
            }

            // Phase 3: Launch valid iframe URL
            const url = new URL('/' + paymentLink, this.iframeUrl);
            url.searchParams.set('widget', 'true');
            url.searchParams.set('theme', theme);
            iframe.removeAttribute('srcdoc');
            iframe.src = url.toString();
        },

        _setupMessageListener: function(config) {
            const boundListener = (event) => {
                // Ensure messages come from our iframe origin (prevent spoofing)
                if (!event.origin.includes('fiddupay.com') && event.origin !== this.iframeUrl) return;

                const data = event.data;
                if (!data || typeof data !== 'object') return;

                if (data.type === 'FIDDUPAY_SUCCESS') {
                    if (typeof config.onSuccess === 'function') {
                        config.onSuccess(data.payload);
                    }
                    if (config.autoCloseOnSuccess !== false) {
                        setTimeout(() => this.close(), 2000);
                    }
                } else if (data.type === 'FIDDUPAY_ERROR') {
                    if (typeof config.onError === 'function') {
                        config.onError(data.payload);
                    }
                } else if (data.type === 'FIDDUPAY_CLOSE') {
                    this.close();
                } else if (data.type === 'FIDDUPAY_CANCEL') {
                    if (typeof config.onCancel === 'function') {
                        config.onCancel();
                    }
                    this.close();
                }
            };
            
            this._messageListener = boundListener;
            window.addEventListener('message', boundListener);
        },

        close: function() {
            const overlay = document.getElementById('fiddupay-overlay');
            if (overlay) {
                overlay.classList.remove('active');
                setTimeout(() => {
                    overlay.remove();
                    document.body.style.overflow = '';
                }, 300);
            }
            if (this._messageListener) {
                window.removeEventListener('message', this._messageListener);
                this._messageListener = null;
            }
        }
    };

    FidduPay.init();
    window.FidduPay = FidduPay;
})(window);
