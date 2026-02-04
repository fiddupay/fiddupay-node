import React, { useState, useEffect, useRef } from 'react';
import { useParams, useLocation } from 'react-router-dom';
import styles from '@/styles/pages/DocsPage.module.css';
import mobileStyles from '@/styles/pages/DocsPageMobile.module.css';
import CodeSnippet from '../components/docs/CodeSnippet';
import DocsSidebar from '../components/docs/DocsSidebar';
import ApiSection from '../components/docs/ApiSection';
import { API_DATA, Endpoint } from './docs/ApiData';

const DocsPage: React.FC = () => {
  const { sectionId } = useParams<{ sectionId: string }>();
  const location = useLocation();
  const [activeSection, setActiveSection] = useState(sectionId || 'getting-started');
  const sectionRefs = useRef<{ [key: string]: HTMLDivElement | null }>({});
  const isScrollingRef = useRef(false);

  // Scroll to section/hash on mount
  useEffect(() => {
    if (sectionId) {
      setActiveSection(sectionId);
      // Allow DOM to render then scroll
      setTimeout(() => {
        const hash = location.hash.replace('#', '');
        const targetId = hash || sectionId;
        const element = document.getElementById(targetId);
        if (element) {
          isScrollingRef.current = true;
          element.scrollIntoView({ behavior: 'smooth' });
          setTimeout(() => { isScrollingRef.current = false; }, 1000);
        }
      }, 100);
    }
  }, [sectionId, location.hash]);

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        // Find the "most visible" section
        let maxRatio = 0;
        let bestTarget = '';

        entries.forEach((entry) => {
          if (entry.intersectionRatio > maxRatio) {
            maxRatio = entry.intersectionRatio;
            bestTarget = entry.target.id;
          }
        });

        if (bestTarget && !isScrollingRef.current) {
          // Check if section is actually a top-level section or an endpoint
          // Usually we want to track the endpoint if it's visible?
          // The original logic tracked section IDs. 
          // Let's assume sectionRefs track actual Sections or Endpoints.

          // To stick to the requirement: update URL
          // Avoid re-triggering navigation if it matches
          if (activeSection !== bestTarget) {
            setActiveSection(bestTarget);
            // Update URL silently to avoid router thrashing
            window.history.replaceState(null, '', `/docs/${bestTarget}`);
          }
        }
      },
      { threshold: [0.1, 0.5, 0.8], rootMargin: '-80px 0px -20% 0px' }
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
