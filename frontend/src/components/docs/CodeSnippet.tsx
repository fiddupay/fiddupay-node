import React, { useState } from 'react';
import styles from '@/styles/components/docs/CodeSnippet.module.css';

interface CodeSnippetProps {
    request?: {
        curl: string;
        node: string;
    };
    response?: string;
    method?: string;
    path?: string;
}

const CodeSnippet: React.FC<CodeSnippetProps> = ({ request, response, method, path }) => {
    const [activeLanguage] = useState<'curl' | 'node'>('curl');

    const copyToClipboard = (text: string) => {
        navigator.clipboard.writeText(text);
    };

    const renderCodeWithLineNumbers = (code: string) => {
        const lines = code.trim().split('\n');
        return (
            <div className={styles.codeGrid}>
                <div className={styles.lineNumbers}>
                    {lines.map((_, i) => (
                        <div key={i}>{i + 1}</div>
                    ))}
                </div>
                <div className={styles.codeContent}>
                    {lines.map((line, i) => (
                        <div key={i}>{highlightSyntax(line)}</div>
                    ))}
                </div>
            </div>
        );
    };

    // Basic syntax highlighting helper
    const highlightSyntax = (line: string) => {
        // This is a naive implementation. For production, use PrismJS or similar.
        // It wraps strings in green, keys in blue.
        const parts = line.split(/(".*?"|'.*?'|\b\d+\b|\btrue\b|\bfalse\b)/g);
        return parts.map((part, index) => {
            if (part.startsWith('"') || part.startsWith("'")) {
                return <span key={index} className={styles.string}>{part}</span>;
            } else if (!isNaN(Number(part)) && part.trim() !== '') {
                return <span key={index} className={styles.number}>{part}</span>;
            } else if (part === 'true' || part === 'false') {
                return <span key={index} className={styles.boolean}>{part}</span>;
            } else if (part.trim().endsWith(':')) {
                return <span key={index} className={styles.key}>{part}</span>;
            }
            return <span key={index}>{part}</span>;
        });
    };

    if (!request && !response) return null;

    return (
        <div className={styles.container}>
            {/* Request Card */}
            {request && (
                <div className={styles.card}>
                    <div className={styles.header}>
                        <div className={styles.headerLeft}>
                            {method && (
                                <span className={`${styles.methodBadge} ${styles[method.toLowerCase()]}`}>
                                    {method}
                                </span>
                            )}
                            {path && <span className={styles.path}>{path}</span>}
                        </div>
                        <div className={styles.headerRight}>
                            <span className={styles.langLabel}>{activeLanguage === 'curl' ? 'cURL' : 'Node.js'}</span>
                        </div>
                    </div>
                    <div className={styles.codeArea}>
                        <div className={styles.copyOverlay}>
                            <button
                                className={styles.copyBtn}
                                onClick={() => copyToClipboard(activeLanguage === 'curl' ? request.curl : request.node)}
                            >
                                Copy
                            </button>
                        </div>
                        {renderCodeWithLineNumbers(activeLanguage === 'curl' ? request.curl : request.node)}
                    </div>
                </div>
            )}

            {/* Response Card */}
            {response && (
                <div className={styles.card}>
                    <div className={styles.header}>
                        <span className={styles.headerTitle}>Sample Response</span>
                        <div className={styles.headerRight}>
                            <span className={styles.statusLabel}>200 OK</span>
                        </div>
                    </div>
                    <div className={styles.codeArea}>
                        <div className={styles.copyOverlay}>
                            <button
                                className={styles.copyBtn}
                                onClick={() => copyToClipboard(response)}
                            >
                                Copy
                            </button>
                        </div>
                        {renderCodeWithLineNumbers(response)}
                    </div>
                </div>
            )}
        </div>
    );
};

export default CodeSnippet;
