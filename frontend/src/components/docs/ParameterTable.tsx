import React from 'react';
import styles from './DocsPage.module.css';
import { Parameter } from '../../pages/docs/ApiData';

interface ParameterTableProps {
    title: string;
    parameters: Parameter[];
}

const ParameterTable: React.FC<ParameterTableProps> = ({ title, parameters }) => {
    if (!parameters || parameters.length === 0) return null;

    return (
        <>
            <h3>{title}</h3>
            <div className={styles.tableContainer}>
                <table className={styles.paramTable}>
                    <thead>
                        <tr>
                            <th>Parameter</th>
                            <th>Description</th>
                        </tr>
                    </thead>
                    <tbody>
                        {parameters.map((p) => (
                            <tr key={p.name}>
                                <td>
                                    <div className={styles.paramName}>
                                        {p.name}
                                        {p.required && <span className={styles.required}>Required</span>}
                                    </div>
                                    <div className={styles.paramType}>{p.type}</div>
                                </td>
                                <td className={styles.paramDesc}>{p.description}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </>
    );
};

export default ParameterTable;
