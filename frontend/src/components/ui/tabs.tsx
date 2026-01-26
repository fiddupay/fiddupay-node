export const Tabs = ({ children, className = '', value, defaultValue, onValueChange, ...props }: any) => (
  <div className={className} {...props}>{children}</div>
)

export const TabsList = ({ children, className = '' }: { children: React.ReactNode; className?: string }) => (
  <div className={`flex ${className}`}>{children}</div>
)

export const TabsTrigger = ({ children, className = '', value, ...props }: any) => (
  <button className={`px-4 py-2 ${className}`} {...props}>{children}</button>
)

export const TabsContent = ({ children, className = '', value, ...props }: any) => (
  <div className={className} {...props}>{children}</div>
)
