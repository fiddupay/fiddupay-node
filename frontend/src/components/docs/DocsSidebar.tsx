import React from 'react';
import styles from '@/styles/pages/DocsPage.module.css';
import { DocSection } from '../../pages/docs/ApiData';

interface DocsSidebarProps {
    apiData: DocSection[];
    activeSection: string;
    scrollToSection: (id: string) => void;
}

const DocsSidebar: React.FC<DocsSidebarProps> = ({ apiData, activeSection, scrollToSection }) => {
    return (
        <aside className={styles.sidebar}>
            <div className={styles.sidebarSection}>
                <div className={styles.sidebarTitle}>API Reference</div>
                {apiData.map((section) => (
                    <div key={section.id}>
                        <a
                            href={`#${section.id}`}
                            className={`${styles.sidebarLink} ${activeSection === section.id ? styles.active : ''}`}
                            onClick={(e) => {
                                e.preventDefault();
                                scrollToSection(section.id);
                            }}
                        >
                            {section.title}
                        </a>
                        {section.endpoints.map((endpoint) => (
                            <a
                                key={endpoint.id}
                                href={`#${endpoint.id}`}
                                className={`${styles.sidebarLink} ${styles.subLink} ${activeSection === endpoint.id ? styles.active : ''}`}
                                style={{ paddingLeft: '40px', fontSize: '12px', opacity: 0.8 }}
                                onClick={(e) => {
                                    e.preventDefault();
                                    scrollToSection(endpoint.id);
                                }}
                            >
                                {endpoint.title}
                            </a>
                        ))}
                    </div>
                ))}
            </div>

            <div className={styles.sidebarSection}>
                <div className={styles.sidebarTitle}>Resources</div>
                <a href="https://github.com/fiddupay/fiddupay-node" target="_blank" rel="noopener noreferrer" className={styles.sidebarLink}>
                    <i className="fab fa-github" style={{ marginRight: '8px' }}></i> GitHub Source
                </a>
                <a href="https://www.npmjs.com/package/@fiddupay/fiddupay-node" target="_blank" rel="noopener noreferrer" className={styles.sidebarLink}>
                    <i className="fab fa-npm" style={{ marginRight: '8px' }}></i> Node.js SDK
                </a>
            </div>
        </aside>
    );
};

export default DocsSidebar;
