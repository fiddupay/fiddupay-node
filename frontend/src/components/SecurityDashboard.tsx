// Security Dashboard Component
// Displays security events, alerts, and monitoring

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { 
  Shield, 
  AlertTriangle, 
  CheckCircle, 
  Clock, 
  Fuel,
  Activity,
  Settings
} from 'lucide-react';

interface SecurityEvent {
  event_type: string;
  ip_address: string;
  details: any;
  severity: 'Low' | 'Medium' | 'High' | 'Critical';
  timestamp: string;
}

interface SecurityAlert {
  id: string;
  alert_type: string;
  message: string;
  sent_at: string;
  acknowledged_at?: string;
}

interface BalanceAlert {
  id: string;
  alert_type: string;
  crypto_type: string;
  current_balance: string;
  threshold: string;
  message: string;
  created_at: string;
  resolved_at?: string;
}

export default function SecurityDashboard() {
  const [securityEvents, setSecurityEvents] = useState<SecurityEvent[]>([]);
  const [securityAlerts, setSecurityAlerts] = useState<SecurityAlert[]>([]);
  const [balanceAlerts, setBalanceAlerts] = useState<BalanceAlert[]>([]);
  const [gasAlerts, setGasAlerts] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>('');

  useEffect(() => {
    loadSecurityData();
  }, []);

  const loadSecurityData = async () => {
    try {
      setLoading(true);
      
      // Load security events
      const eventsResponse = await fetch('/api/v1/security/events?limit=20', {
        headers: { 'Authorization': `Bearer ${localStorage.getItem('api_key')}` }
      });
      const eventsData = await eventsResponse.json();
      setSecurityEvents(eventsData.events || []);

      // Load security alerts
      const alertsResponse = await fetch('/api/v1/security/alerts?limit=10', {
        headers: { 'Authorization': `Bearer ${localStorage.getItem('api_key')}` }
      });
      const alertsData = await alertsResponse.json();
      setSecurityAlerts(alertsData.alerts || []);

      // Load balance alerts
      const balanceResponse = await fetch('/api/v1/security/balance-alerts?active_only=true', {
        headers: { 'Authorization': `Bearer ${localStorage.getItem('api_key')}` }
      });
      const balanceData = await balanceResponse.json();
      setBalanceAlerts(balanceData.alerts || []);

      // Load gas balance check
      const gasResponse = await fetch('/api/v1/security/gas-check', {
        headers: { 'Authorization': `Bearer ${localStorage.getItem('api_key')}` }
      });
      const gasData = await gasResponse.json();
      setGasAlerts(gasData.gas_alerts || []);

    } catch (err) {
      setError('Failed to load security data');
    } finally {
      setLoading(false);
    }
  };

  const acknowledgeAlert = async (alertId: string) => {
    try {
      await fetch(`/api/v1/security/alerts/${alertId}/acknowledge`, {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${localStorage.getItem('api_key')}` }
      });
      loadSecurityData();
    } catch (err) {
      setError('Failed to acknowledge alert');
    }
  };

  const resolveBalanceAlert = async (alertId: string) => {
    try {
      await fetch(`/api/v1/security/balance-alerts/${alertId}/resolve`, {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${localStorage.getItem('api_key')}` }
      });
      loadSecurityData();
    } catch (err) {
      setError('Failed to resolve alert');
    }
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'Critical': return 'bg-red-100 text-red-800 border-red-200';
      case 'High': return 'bg-orange-100 text-orange-800 border-orange-200';
      case 'Medium': return 'bg-yellow-100 text-yellow-800 border-yellow-200';
      default: return 'bg-blue-100 text-blue-800 border-blue-200';
    }
  };

  const formatTimestamp = (timestamp: string) => {
    return new Date(timestamp).toLocaleString();
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center p-8">
        <Clock className="h-6 w-6 animate-spin mr-2" />
        Loading security data...
      </div>
    );
  }

  return (
    <div className="max-w-6xl mx-auto p-6 space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Security Dashboard
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Tabs defaultValue="overview" className="space-y-4">
            <TabsList className="grid w-full grid-cols-4">
              <TabsTrigger value="overview">Overview</TabsTrigger>
              <TabsTrigger value="events">Security Events</TabsTrigger>
              <TabsTrigger value="alerts">Alerts</TabsTrigger>
              <TabsTrigger value="monitoring">Monitoring</TabsTrigger>
            </TabsList>

            <TabsContent value="overview" className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                {/* Security Status */}
                <Card>
                  <CardContent className="p-4">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm text-gray-600">Security Status</p>
                        <p className="text-2xl font-bold text-green-600">Secure</p>
                      </div>
                      <Shield className="h-8 w-8 text-green-600" />
                    </div>
                  </CardContent>
                </Card>

                {/* Active Alerts */}
                <Card>
                  <CardContent className="p-4">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm text-gray-600">Active Alerts</p>
                        <p className="text-2xl font-bold text-orange-600">
                          {securityAlerts.filter(a => !a.acknowledged_at).length + balanceAlerts.length}
                        </p>
                      </div>
                      <AlertTriangle className="h-8 w-8 text-orange-600" />
                    </div>
                  </CardContent>
                </Card>

                {/* Gas Warnings */}
                <Card>
                  <CardContent className="p-4">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm text-gray-600">Gas Warnings</p>
                        <p className="text-2xl font-bold text-blue-600">{gasAlerts.length}</p>
                      </div>
                      <Fuel className="h-8 w-8 text-blue-600" />
                    </div>
                  </CardContent>
                </Card>
              </div>

              {/* Recent Critical Events */}
              {securityEvents.filter(e => e.severity === 'Critical' || e.severity === 'High').length > 0 && (
                <Alert className="border-red-200 bg-red-50">
                  <AlertTriangle className="h-4 w-4" />
                  <AlertDescription>
                    <strong>High Priority Events Detected</strong>
                    <div className="mt-2 space-y-1">
                      {securityEvents
                        .filter(e => e.severity === 'Critical' || e.severity === 'High')
                        .slice(0, 3)
                        .map((event, index) => (
                          <div key={index} className="text-sm">
                            • {event.event_type} from {event.ip_address} ({formatTimestamp(event.timestamp)})
                          </div>
                        ))}
                    </div>
                  </AlertDescription>
                </Alert>
              )}
            </TabsContent>

            <TabsContent value="events" className="space-y-4">
              <div className="space-y-3">
                {securityEvents.map((event, index) => (
                  <Card key={index}>
                    <CardContent className="p-4">
                      <div className="flex items-start justify-between">
                        <div className="flex-1">
                          <div className="flex items-center gap-2 mb-2">
                            <Badge className={getSeverityColor(event.severity)}>
                              {event.severity}
                            </Badge>
                            <span className="font-medium">{event.event_type}</span>
                          </div>
                          <p className="text-sm text-gray-600 mb-1">
                            IP: {event.ip_address}
                          </p>
                          <p className="text-xs text-gray-500">
                            {formatTimestamp(event.timestamp)}
                          </p>
                        </div>
                        <Activity className="h-5 w-5 text-gray-400" />
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </TabsContent>

            <TabsContent value="alerts" className="space-y-4">
              {/* Security Alerts */}
              <div>
                <h3 className="text-lg font-medium mb-3">Security Alerts</h3>
                <div className="space-y-3">
                  {securityAlerts.map((alert) => (
                    <Card key={alert.id}>
                      <CardContent className="p-4">
                        <div className="flex items-start justify-between">
                          <div className="flex-1">
                            <div className="flex items-center gap-2 mb-2">
                              <Badge variant={alert.acknowledged_at ? 'secondary' : 'destructive'}>
                                {alert.alert_type}
                              </Badge>
                              {alert.acknowledged_at ? (
                                <CheckCircle className="h-4 w-4 text-green-600" />
                              ) : (
                                <AlertTriangle className="h-4 w-4 text-red-600" />
                              )}
                            </div>
                            <p className="text-sm mb-2">{alert.message}</p>
                            <p className="text-xs text-gray-500">
                              {formatTimestamp(alert.sent_at)}
                            </p>
                          </div>
                          {!alert.acknowledged_at && (
                            <Button
                              size="sm"
                              variant="outline"
                              onClick={() => acknowledgeAlert(alert.id)}
                            >
                              Acknowledge
                            </Button>
                          )}
                        </div>
                      </CardContent>
                    </Card>
                  ))}
                </div>
              </div>

              {/* Balance Alerts */}
              <div>
                <h3 className="text-lg font-medium mb-3">Balance Alerts</h3>
                <div className="space-y-3">
                  {balanceAlerts.map((alert) => (
                    <Card key={alert.id}>
                      <CardContent className="p-4">
                        <div className="flex items-start justify-between">
                          <div className="flex-1">
                            <div className="flex items-center gap-2 mb-2">
                              <Badge variant="secondary">{alert.alert_type}</Badge>
                              <span className="text-sm font-medium">{alert.crypto_type}</span>
                            </div>
                            <p className="text-sm mb-2">{alert.message}</p>
                            <p className="text-xs text-gray-500">
                              Current: {alert.current_balance} | Threshold: {alert.threshold}
                            </p>
                            <p className="text-xs text-gray-500">
                              {formatTimestamp(alert.created_at)}
                            </p>
                          </div>
                          {!alert.resolved_at && (
                            <Button
                              size="sm"
                              variant="outline"
                              onClick={() => resolveBalanceAlert(alert.id)}
                            >
                              Resolve
                            </Button>
                          )}
                        </div>
                      </CardContent>
                    </Card>
                  ))}
                </div>
              </div>
            </TabsContent>

            <TabsContent value="monitoring" className="space-y-4">
              {/* Gas Balance Monitoring */}
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Fuel className="h-5 w-5" />
                    Gas Balance Monitoring
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  {gasAlerts.length > 0 ? (
                    <div className="space-y-3">
                      {gasAlerts.map((alert, index) => (
                        <Alert key={index} className="border-orange-200 bg-orange-50">
                          <Fuel className="h-4 w-4" />
                          <AlertDescription>
                            <strong>Low Gas Balance:</strong> {alert.message}
                          </AlertDescription>
                        </Alert>
                      ))}
                    </div>
                  ) : (
                    <div className="text-center py-8 text-gray-500">
                      <CheckCircle className="h-12 w-12 mx-auto mb-2 text-green-600" />
                      <p>All gas balances are sufficient</p>
                    </div>
                  )}
                </CardContent>
              </Card>

              {/* Security Settings */}
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Settings className="h-5 w-5" />
                    Security Settings
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div className="flex items-center justify-between p-3 border rounded">
                      <span>Withdrawal Confirmations</span>
                      <Badge variant="secondary">Enabled</Badge>
                    </div>
                    <div className="flex items-center justify-between p-3 border rounded">
                      <span>Email Alerts</span>
                      <Badge variant="secondary">Enabled</Badge>
                    </div>
                    <div className="flex items-center justify-between p-3 border rounded">
                      <span>Large Withdrawal Threshold</span>
                      <Badge variant="secondary">$10,000</Badge>
                    </div>
                    <div className="flex items-center justify-between p-3 border rounded">
                      <span>Rate Limiting</span>
                      <Badge variant="secondary">Active</Badge>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </TabsContent>
          </Tabs>

          {error && (
            <Alert className="mt-4" variant="destructive">
              <AlertTriangle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
