import React from 'react';
import { Card, CardHeader, CardTitle, CardContent } from './card';
import { Badge } from './badge';
import { 
  MdShield, 
  MdWarning, 
  MdCheckCircle, 
  MdOutlineCircle 
} from 'react-icons/md';
import { User } from '@/types';

interface TrustScoreWidgetProps {
  user: User | null;
  className?: string;
}

export const TrustScoreWidget: React.FC<TrustScoreWidgetProps> = ({ user, className = "" }) => {
  if (!user || !user.trust_score) return null;

  const { score, tier, identity_verified, social_verified, business_verified } = user.trust_score;

  const getTierColor = (t: string) => {
    switch (t) {
      case 'Gold': return 'from-yellow-400 to-amber-600';
      case 'Silver': return 'from-slate-300 to-slate-500';
      default: return 'from-orange-400 to-orange-700';
    }
  };

  const getScoreColor = (s: number) => {
    if (s >= 80) return 'text-green-500';
    if (s >= 60) return 'text-blue-500';
    if (s >= 40) return 'text-yellow-500';
    return 'text-red-500';
  };

  const checklistItems = [
    { label: 'Identity Agent (NIN/BVN)', status: identity_verified, points: 60 },
    { label: 'Social Signal Agent (Handles)', status: social_verified, points: 20 },
    { label: 'Reputation Agent (History)', status: business_verified, points: 20 },
  ];

  return (
    <Card className={`overflow-hidden border-none bg-white/5 backdrop-blur-xl shadow-2xl ${className}`}>
      <div className={`h-1.5 w-full bg-gradient-to-r ${getTierColor(tier)}`} />
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium text-gray-400 uppercase tracking-wider flex items-center gap-2">
            <MdShield className="w-4 h-4 text-primary" />
            Trust Intelligence Layer
          </CardTitle>
          <Badge className={`bg-gradient-to-br ${getTierColor(tier)} text-white border-none px-3 py-1 font-bold shadow-lg text-[10px]`}>
            {tier} TIER
          </Badge>
        </div>
      </CardHeader>
      <CardContent>
        <div className="flex items-end gap-4 mb-6">
          <div className={`text-6xl font-black tracking-tighter ${getScoreColor(score)}`}>
            {score}<span className="text-2xl opacity-50">%</span>
          </div>
          <div className="pb-2">
            <div className="text-xs text-gray-500 uppercase font-bold">Network Reputation</div>
            <div className="text-sm text-gray-300 font-medium">Agent Consensus Level</div>
          </div>
        </div>

        <div className="space-y-3">
          {checklistItems.map((item, idx) => (
            <div key={idx} className="flex items-center justify-between group">
              <div className="flex items-center gap-3">
                {item.status ? (
                  <MdCheckCircle className="w-5 h-5 text-green-500" />
                ) : (
                  <MdOutlineCircle className="w-5 h-5 text-gray-600 group-hover:text-amber-500 transition-colors" />
                )}
                <span className={`text-sm ${item.status ? 'text-gray-200' : 'text-gray-500'}`}>
                  {item.label}
                </span>
              </div>
              <span className={`text-xs font-mono ${item.status ? 'text-green-500/50' : 'text-gray-600'}`}>
                +{item.points}
              </span>
            </div>
          ))}
        </div>

        <div className="mt-8 pt-6 border-t border-white/5">
            <div className="flex items-center justify-between text-[10px] uppercase font-black tracking-widest text-gray-500 mb-4">
                <span>Swarm Pulse</span>
                <span className="text-primary">LIVE SIGNAL</span>
            </div>
            <div className="flex gap-1 h-1">
                {[...Array(12)].map((_, i) => (
                    <div key={i} className={`flex-1 rounded-full ${i < (score / 8) ? 'bg-primary' : 'bg-white/10'} ${i === Math.floor(score / 8) ? 'animate-pulse' : ''}`} />
                ))}
            </div>
        </div>

        {score < 100 && (
          <div className="mt-6 p-3 rounded-lg bg-primary/5 border border-primary/20 flex gap-3 items-start">
            <MdWarning className="w-4 h-4 text-primary shrink-0 mt-0.5" />
            <p className="text-[11px] text-gray-400 leading-relaxed">
              Unlock 0-fee interoperability and higher limits by feeding the Swarm more signals. 
              {score < 60 ? " Start with Identity Agent verification." : " Link Social Signals to reach Gold status."}
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  );
};
