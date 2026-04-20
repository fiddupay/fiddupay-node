import React from 'react';
import styles from '@/styles/components/docs/ApiSection.module.css';
import CodeSnippet from './CodeSnippet';
import { DocSection, Endpoint, SubSection } from '../../pages/docs/ApiData';
import ParameterTable from './ParameterTable';
import MarkdownText from './MarkdownText';
import DocSubSection from './DocSubSection';

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
            {section.description && (
                <div className={styles.lead}>
                    <MarkdownText text={section.description} />
                </div>
            )}

            {section.endpoints.map((endpoint: Endpoint) => (
                <div
                    key={endpoint.id}
                    id={endpoint.id}
                    ref={(el) => (sectionRefs.current[endpoint.id] = el)}
                    style={{ marginTop: '64px' }}
                >
                    <h2>{endpoint.title}</h2>
                    <div className={styles.lead}>
                        <MarkdownText text={endpoint.description} />
                    </div>
                    <div style={{ display: 'flex', alignItems: 'center', marginBottom: '16px', flexWrap: 'wrap', gap: '8px' }}>
                        <span className={`${styles.methodBadge} ${styles[endpoint.method.toLowerCase()]}`}>
                            {endpoint.method}
                        </span>
                        <span className={styles.endpointPath} style={{ wordBreak: 'break-all' }}>{endpoint.path}</span>
                        {endpoint.deprecated && (
                            <span className={styles.deprecatedBadge}>DEPRECATED</span>
                        )}
                    </div>

                    <div className={styles.mobileCodeSnippet}>
                        {endpoint.request && (
                            <CodeSnippet
                                request={endpoint.request}
                                response={endpoint.response}
                                method={endpoint.method}
                                path={endpoint.path}
                            />
                        )}
                    </div>

                    <ParameterTable title="Query Parameters" parameters={endpoint.parameters || []} />
                    <ParameterTable title="Request Body" parameters={endpoint.body || []} />

                    {endpoint.subSections && endpoint.subSections.length > 0 && (
                        <div className={styles.subSectionsContainer}>
                            {endpoint.subSections.map((sub: SubSection, i: number) => (
                                <DocSubSection key={i} subSection={sub} />
                            ))}
                        </div>
                    )}
                </div>
            ))}
        </div>
    );
};

export default ApiSection;
