import React, { useState, useEffect, useRef } from 'react';
import styles from '@/styles/pages/DocsPage.module.css';
import mobileStyles from '@/styles/pages/DocsPageMobile.module.css';
import CodeSnippet from '../components/docs/CodeSnippet';
import DocsSidebar from '../components/docs/DocsSidebar';
import ApiSection from '../components/docs/ApiSection';
import { API_DATA, Endpoint } from './docs/ApiData';

const DocsPage: React.FC = () => {
  const [activeSection, setActiveSection] = useState('getting-started');
  const sectionRefs = useRef<{ [key: string]: HTMLDivElement | null }>({});

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            setActiveSection(entry.target.id);
          }
        });
      },
      { threshold: 0.2, rootMargin: '-80px 0px -20% 0px' }
    );

    Object.values(sectionRefs.current).forEach((ref) => {
      if (ref) observer.observe(ref);
    });

    return () => observer.disconnect();
  }, []);

  const scrollToSection = (id: string) => {
    const element = document.getElementById(id);
    if (element) {
      element.scrollIntoView({ behavior: 'smooth' });
    }
  };

  return (
    <div className={`${styles.docsPage} ${mobileStyles.docsPage}`}>
      {/* Column 1: Navigation */}
      <aside className={`${styles.sidebar} ${mobileStyles.sidebar}`}>
        <DocsSidebar
          apiData={API_DATA}
          activeSection={activeSection}
          scrollToSection={scrollToSection}
        />
      </aside>

      {/* Column 2 & 3 Wrap */}
      <div className={`${styles.contentArea} ${mobileStyles.contentArea}`}>
        <main className={`${styles.mainContent} ${mobileStyles.mainContent}`}>
          {API_DATA.map((section) => (
            <ApiSection
              key={section.id}
              section={section}
              sectionRefs={sectionRefs}
            />
          ))}

          <div className={styles.infoBox}>
            <p>
              Looking for older documentation? Check our <a href="https://github.com/fiddupay/fiddupay-node/releases" target="_blank" rel="noopener noreferrer">release notes</a> for version changes and migration guides.
            </p>
          </div>
        </main>

        {/* Column 3: Code Samples */}
        {(() => {
          const item = API_DATA.flatMap(s => [s, ...s.endpoints]).find(e => e.id === activeSection);
          const hasCode = item && (item as Endpoint).request;

          return (
            <aside
              className={`${styles.codeColumn} ${mobileStyles.codeColumn}`}
              style={{ display: hasCode ? 'block' : 'none' }}
            >
              <div className={`${styles.codeSticky} ${mobileStyles.codeSticky}`}>
                {hasCode && (
                  <div key={activeSection}>
                    <CodeSnippet
                      request={(item as Endpoint).request}
                      response={(item as Endpoint).response}
                      method={(item as Endpoint).method}
                      path={(item as Endpoint).path}
                    />
                  </div>
                )}
              </div>
            </aside>
          );
        })()}
      </div>
    </div>
  );
};

export default DocsPage;
