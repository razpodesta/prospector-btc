'use client';

import { useState, useEffect } from 'react';
import { Lock, Unlock, ArrowRight, Loader2, AlertTriangle } from 'lucide-react';
import { apiClient } from '@prospector/api-client'; // Usamos el cliente configurado

export function AdminGuard({ children }: { children: React.ReactNode }) {
  const [isUnlocked, setIsUnlocked] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [key, setKey] = useState('');
  const [error, setError] = useState('');

  // Verificación real contra el servidor
  const checkKey = async () => {
    setIsLoading(true);
    setError('');

    try {
      // Intentamos acceder a un endpoint protegido ligero (/status es público, usamos uno admin)
      // O simplemente intentamos listar identidades. Si el token es malo, dará 401.
      // Aquí, inyectamos el token temporalmente en la petición para probarlo.

      await apiClient.get('/admin/identities', {
        headers: { Authorization: `Bearer ${key}` }
      });

      // Si pasa (200 OK), el token es válido.
      sessionStorage.setItem('ADMIN_SESSION_TOKEN', key);
      // Forzamos un reload suave o actualización de estado global si fuera necesario
      setIsUnlocked(true);
    } catch (e: any) {
      console.error("Auth Failed", e);
      setError('ACCESS DENIED: INVALID TOKEN OR UNAUTHORIZED');
      sessionStorage.removeItem('ADMIN_SESSION_TOKEN');
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    const storedToken = sessionStorage.getItem('ADMIN_SESSION_TOKEN');
    if (storedToken) {
      setKey(storedToken);
      // Opcional: Podríamos re-verificar aquí automáticamente
      setIsUnlocked(true);
    }
  }, []);

  if (isUnlocked) return <>{children}</>;

  return (
    <div className="min-h-screen bg-black flex items-center justify-center p-4 font-mono">
      <div className="max-w-md w-full bg-[#0a0a0a] border border-slate-800 p-8 rounded-xl shadow-2xl relative overflow-hidden group">
        {/* Efecto de Scanline */}
        <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-transparent via-emerald-500 to-transparent opacity-50 group-hover:animate-scan" />

        <div className="text-center mb-8">
          <div className="mx-auto w-16 h-16 bg-slate-900 rounded-full flex items-center justify-center mb-4 border border-slate-700 shadow-[0_0_15px_rgba(0,0,0,0.5)]">
            {isLoading ? <Loader2 className="w-8 h-8 text-emerald-500 animate-spin" /> : <Lock className="w-8 h-8 text-slate-400" />}
          </div>
          <h1 className="text-xl font-bold text-white tracking-widest uppercase">Restricted Area</h1>
          <p className="text-xs text-slate-500 mt-2">AUTHORIZATION REQUIRED // LEVEL 5</p>
        </div>

        <div className="space-y-4">
          <div className="relative">
            <input
              type="password"
              value={key}
              onChange={(e) => { setKey(e.target.value); setError(''); }}
              placeholder="ENTER MASTER KEY"
              disabled={isLoading}
              className="w-full bg-black border border-slate-700 text-emerald-500 text-center py-3 px-4 rounded focus:outline-none focus:border-emerald-500 transition-colors uppercase placeholder:text-slate-800 disabled:opacity-50"
              onKeyDown={(e) => e.key === 'Enter' && checkKey()}
            />
          </div>

          <button
            onClick={checkKey}
            disabled={isLoading || key.length < 5}
            className="w-full bg-emerald-900/20 hover:bg-emerald-900/40 text-emerald-500 border border-emerald-900/50 py-3 rounded flex items-center justify-center gap-2 transition-all uppercase text-sm font-bold tracking-wider group disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isLoading ? 'Verifying...' : 'Authenticate'}
            {!isLoading && <ArrowRight className="w-4 h-4 group-hover:translate-x-1 transition-transform" />}
          </button>

          {error && (
            <div className="flex items-center justify-center gap-2 text-red-500 text-xs mt-4 animate-pulse">
              <AlertTriangle className="w-3 h-3" />
              {error}
            </div>
          )}
        </div>

        <div className="mt-8 text-[10px] text-center text-slate-700">
            SECURE CONNECTION ESTABLISHED TO ORCHESTRATOR
        </div>
      </div>
    </div>
  );
}
