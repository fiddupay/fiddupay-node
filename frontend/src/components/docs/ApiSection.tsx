import React from 'react';
import styles from '../../pages/DocsPage.module.css';
import { DocSection } from '../../pages/docs/ApiData';
import ParameterTable from './ParameterTable';

interface ApiSectionProps {
    section: DocSection;
    sectionRefs: React.MutableRefObject<{ [key: string]: HTMLDivElement | null }>;
}

const ApiSection: React.FC<ApiSectionProps> = ({ section, sectionRefs }) => {
    return (
        <div
            id={section.id}
            ref={(el) => (sectionRefs.current[section.id] = el)}
            className={styles.section}
        >
            <h1>{section.title}</h1>
            {section.description && <p className={styles.lead}>{section.description}</p>}

            {section.endpoints.map((endpoint) => (
                <div
                    key={endpoint.id}
                    id={endpoint.id}
                    ref={(el) => (sectionRefs.current[endpoint.id] = el)}
                    style={{ marginTop: '64px' }}
                >
                    <div style={{ display: 'flex', alignItems: 'center', marginBottom: '16px' }}>
                        <span className={`${styles.methodBadge} ${styles[endpoint.method.toLowerCase()]}`}>
                            {endpoint.method}
                        </span>
                        <span className={styles.endpointPath}>{endpoint.path}</span>
                    </div>
                    <h2>{endpoint.title}</h2>
                    <p className={styles.lead}>{endpoint.description}</p>

                    <ParameterTable title="Query Parameters" parameters={endpoint.parameters || []} />
                    <ParameterTable title="Request Body" parameters={endpoint.body || []} />
                </div>
            ))}
        </div>
    );
};

export default ApiSection;
