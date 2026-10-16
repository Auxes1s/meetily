'use client';

import React from 'react';
import { useSidebar } from '@/components/Sidebar/SidebarProvider';

interface MainContentProps {
  children: React.ReactNode;
}

const MainContent: React.FC<MainContentProps> = ({ children }) => {
  const { isCollapsed } = useSidebar();

  return (
    <main
      id="meetily-main"
      tabIndex={-1}
      className={`app-main flex-1 min-h-0 min-w-0 h-full overflow-hidden transition-[margin] duration-200 ${
        isCollapsed ? 'ml-16' : 'ml-64'
      }`}
    >
      <div className="min-h-0 min-w-0 h-full w-full max-w-full overflow-hidden">
        {children}
      </div>
    </main>
  );
};

export default MainContent;
