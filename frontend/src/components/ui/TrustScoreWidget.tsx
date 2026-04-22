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
    { label: 'Identity Verification (NIN/BVN)', status: identity_verified, points: 60 },
    { label: 'Social Handles Linked', status: social_verified, points: 20 },
    { label: 'Business Registration (Tier 2)', status: business_verified, points: 20 },
  ];

  return (
    <Card className={`overflow-hidden border-none bg-white/5 backdrop-blur-xl shadow-2xl ${className}`}>
      <div className={`h-1.5 w-full bg-gradient-to-r ${getTierColor(tier)}`} />
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium text-gray-400 uppercase tracking-wider flex items-center gap-2">
            <MdShield className="w-4 h-4" />
            Trust Intelligence
          </CardTitle>
          <Badge className={`bg-gradient-to-br ${getTierColor(tier)} text-white border-none px-3 py-1 font-bold shadow-lg`}>
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
            <div className="text-xs text-gray-500 uppercase font-bold">Health Score</div>
            <div className="text-sm text-gray-300 font-medium">Self-Governing Level</div>
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

        {score < 100 && (
          <div className="mt-8 p-3 rounded-lg bg-amber-500/10 border border-amber-500/20 flex gap-3 items-start">
            <MdWarning className="w-5 h-5 text-amber-500 shrink-0 mt-0.5" />
            <p className="text-xs text-amber-200/80 leading-relaxed">
              Unlock 0-fee interoperability and higher limits by completing your profile. 
              {score < 60 ? " Start with Identity Verification to reach Silver status immediately." : " Link your social handles to reach Gold."}
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  );
};
