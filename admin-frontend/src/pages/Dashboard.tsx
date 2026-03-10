import React from 'react';
import { CreditCard, Activity, Droplets } from 'lucide-react';

const Dashboard: React.FC = () => {
    return (
        <div className="space-y-6">
            <h1 className="text-2xl font-semibold text-gray-900">Dashboard</h1>

            <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">

                <div className="bg-white overflow-hidden shadow rounded-lg pointer-events-none">
                    <div className="p-5">
                        <div className="flex items-center">
                            <div className="flex-shrink-0">
                                <CreditCard className="h-6 w-6 text-gray-400" aria-hidden="true" />
                            </div>
                            <div className="ml-5 w-0 flex-1">
                                <dl>
                                    <dt className="text-sm font-medium text-gray-500 truncate">Total Processed</dt>
                                    <dd>
                                        <div className="text-lg font-medium text-gray-900">$2,400.00</div>
                                    </dd>
                                </dl>
                            </div>
                        </div>
                    </div>
                </div>

                <div className="bg-white overflow-hidden shadow rounded-lg pointer-events-none">
                    <div className="p-5">
                        <div className="flex items-center">
                            <div className="flex-shrink-0">
                                <Activity className="h-6 w-6 text-gray-400" aria-hidden="true" />
                            </div>
                            <div className="ml-5 w-0 flex-1">
                                <dl>
                                    <dt className="text-sm font-medium text-gray-500 truncate">Active Merchants</dt>
                                    <dd>
                                        <div className="text-lg font-medium text-gray-900">42</div>
                                    </dd>
                                </dl>
                            </div>
                        </div>
                    </div>
                </div>

                <div className="bg-white overflow-hidden shadow rounded-lg pointer-events-none">
                    <div className="p-5">
                        <div className="flex items-center">
                            <div className="flex-shrink-0">
                                <Droplets className="h-6 w-6 text-gray-400" aria-hidden="true" />
                            </div>
                            <div className="ml-5 w-0 flex-1">
                                <dl>
                                    <dt className="text-sm font-medium text-gray-500 truncate">Fees Swept</dt>
                                    <dd>
                                        <div className="text-lg font-medium text-gray-900">$142.50</div>
                                    </dd>
                                </dl>
                            </div>
                        </div>
                    </div>
                </div>

            </div>

            <div className="bg-white shadow rounded-lg p-6">
                <h2 className="text-lg font-medium text-gray-900 mb-4">Welcome to FidduPay Admin</h2>
                <p className="text-sm text-gray-500 mb-4">
                    This is the administrative dashboard. Currently, most features are accessed through the
                    Settings page, including the new Smart Fee Sweeping module.
                </p>
            </div>
        </div>
    );
};

export default Dashboard;
