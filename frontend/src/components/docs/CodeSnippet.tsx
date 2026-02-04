import React, { useState } from 'react';
import styles from '@/styles/components/docs/CodeSnippet.module.css';

interface CodeSnippetProps {
    request?: {
        curl: string;
        node: string;
    };
    response?: string;
}

const CodeSnippet: React.FC<CodeSnippetProps> = ({ request, response }) => {
    const [activeTab, setActiveTab] = useState<'request' | 'response'>('request');
    const [activeLanguage, setActiveLanguage] = useState<'curl' | 'node'>('curl');

    if (!request && !response) return null;

    return (
        <div className={styles.container}>
            <div className={styles.header}>
                <div className={styles.tabs}>
                    {request && (
                        <button
                            className={`${styles.tab} ${activeTab === 'request' ? styles.active : ''}`}
                            onClick={() => setActiveTab('request')}
                        >
                            Request
                        </button>
                    )}
                    {response && (
                        <button
                            className={`${styles.tab} ${activeTab === 'response' ? styles.active : ''}`}
                            onClick={() => setActiveTab('response')}
                        >
                            Response
                        </button>
                    )}
                </div>

                {activeTab === 'request' && request && (
                    <div className={styles.languages}>
                        <button
                            className={`${styles.langBtn} ${activeLanguage === 'curl' ? styles.active : ''}`}
                            onClick={() => setActiveLanguage('curl')}
                        >
                            cURL
                        </button>
                        <button
                            className={`${styles.langBtn} ${activeLanguage === 'node' ? styles.active : ''}`}
                            onClick={() => setActiveLanguage('node')}
                        >
                            Node.js
                        </button>
                    </div>
                )}
            </div>

            <div className={styles.content}>
                {activeTab === 'request' && request ? (
                    <pre className={styles.pre}>
                        <code>{activeLanguage === 'curl' ? request.curl : request.node}</code>
                    </pre>
                ) : (
                    <pre className={styles.pre}>
                        <code>{response}</code>
                    </pre>
                )}

                <button
                    className={styles.copyBtn}
                    onClick={() => {
                        const text = activeTab === 'request' && request
                            ? (activeLanguage === 'curl' ? request.curl : request.node)
                            : response;
                        if (text) navigator.clipboard.writeText(text);
                    }}
                    title="Copy code"
                >
                    <i className="fas fa-copy"></i>
                </button>
            </div>
        </div>
    );
};

export default CodeSnippet;
