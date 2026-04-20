import React from 'react';
import styles from '@/styles/components/docs/ApiSection.module.css';

interface MarkdownTextProps {
    text: string;
}

/**
 * A simple markdown renderer that handles:
 * - `code` inline spans
 * - **bold** text
 * - [links](url)
 * - \n\n line breaks (paragraphs)
 * - > [!IMPORTANT] alerts
 * - 1. 2. 3. ordered lists
 */
const MarkdownText: React.FC<MarkdownTextProps> = ({ text }) => {
    if (!text) return null;

    const renderInlines = (line: string) => {
        return line.split(/(`[^`]+`|\*\*[^*]+\*\*|\[[^\]]+\]\([^)]+\))/g).map((part, i) => {
            if (part.startsWith('`') && part.endsWith('`')) {
                return <code key={i} style={{ 
                    background: 'rgba(255, 255, 255, 0.05)', 
                    padding: '2px 6px', 
                    borderRadius: '4px',
                    fontFamily: 'monospace',
                    color: '#e2e8f0',
                    fontSize: '0.9em'
                }}>{part.slice(1, -1)}</code>;
            }
            if (part.startsWith('**') && part.endsWith('**')) {
                return <strong key={i} style={{ color: '#fff' }}>{part.slice(2, -2)}</strong>;
            }
            
            const linkMatch = part.match(/\[([^\]]+)\]\(([^)]+)\)/);
            if (linkMatch) {
                return (
                    <a 
                        key={i} 
                        href={linkMatch[2]} 
                        target="_blank" 
                        rel="noopener noreferrer"
                        style={{ color: '#6366f1', textDecoration: 'none' }}
                    >
                        {linkMatch[1]}
                    </a>
                );
            }
            return part;
        });
    };

    // Split by double newline for blocks
    const blocks = text.split('\n\n');

    return (
        <>
            {blocks.map((block, bIdx) => {
                const lines = block.split('\n').filter(l => l.trim() !== '');
                
                // Check if it's an alert
                if (block.includes('[!IMPORTANT]') || block.includes('[!NOTE]')) {
                    const isImportant = block.includes('IMPORTANT');
                    const cleanBlock = block
                        .split('\n')
                        .filter(l => !l.includes('[!'))
                        .map(l => l.replace(/^>\s?/, ''))
                        .join('\n\n');

                    return (
                        <div key={bIdx} className={`${styles.alert} ${isImportant ? styles.alertImportant : ''}`}>
                            <div className={styles.alertTitle}>
                                <i className={`fas ${isImportant ? 'fa-exclamation-triangle' : 'fa-info-circle'}`}></i>
                                {isImportant ? 'Important' : 'Note'}
                            </div>
                            <div className={styles.alertContent}>
                                <MarkdownText text={cleanBlock} />
                            </div>
                        </div>
                    );
                }

                // Check if it's an ordered list
                if (lines.length > 0 && /^\d+\.\s/.test(lines[0])) {
                    return (
                        <ol key={bIdx} className={`${styles.docList} ${styles.orderedList}`}>
                            {lines.map((line, lIdx) => (
                                <li key={lIdx}>
                                    {renderInlines(line.replace(/^\d+\.\s+/, ''))}
                                </li>
                            ))}
                        </ol>
                    );
                }

                // Check if it's an unordered list
                if (lines.length > 0 && /^[-*]\s/.test(lines[0])) {
                    return (
                        <ul key={bIdx} className={`${styles.docList} ${styles.unorderedList}`}>
                            {lines.map((line, lIdx) => (
                                <li key={lIdx}>
                                    {renderInlines(line.replace(/^[-*]\s+/, ''))}
                                </li>
                            ))}
                        </ul>
                    );
                }

                // Default paragraph
                return (
                    <p key={bIdx} style={{ marginBottom: bIdx === blocks.length - 1 ? 0 : '1.25rem' }}>
                        {renderInlines(block)}
                    </p>
                );
            })}
        </>
    );
};

export default MarkdownText;
