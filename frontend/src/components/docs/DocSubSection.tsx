import React from 'react';
import styles from '@/styles/components/docs/ApiSection.module.css';
import { SubSection } from '../../pages/docs/ApiData';
import MarkdownText from './MarkdownText';

interface DocSubSectionProps {
    subSection: SubSection;
}

const DocSubSection: React.FC<DocSubSectionProps> = ({ subSection }) => {
    return (
        <div className={styles.subSection}>
            <h4 className={styles.subSectionTitle}>{subSection.title}</h4>
            <div className={styles.subSectionGrid}>
                {subSection.items.map((item, idx) => (
                    <div key={idx} className={styles.subSectionCard}>
                        <div className={styles.cardHeader}>
                            <code className={styles.cardKey}>{item.key}</code>
                        </div>
                        <div className={styles.cardContent}>
                            <MarkdownText text={item.description} />
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
};

export default DocSubSection;
