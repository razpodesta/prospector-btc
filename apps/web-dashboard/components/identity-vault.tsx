'use client';

import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '@prospector/api-client'; // Asumimos cliente configurado
import { Save, RefreshCw, Shield, Clock, Activity, User, Server } from 'lucide-react';

// Tipado Frontend
interface Identity {
  id: string;
  platform: string;
  email: string;
  usage_count: number;
  last_used_at: string | null;
  status: 'active' | 'expired' | 'revoked';
}

export function IdentityVault() {
  const queryClient = useQueryClient();
  const [jsonInput, setJsonInput] = useState('');
  const [emailInput, setEmailInput] = useState('');
  const [platformInput, setPlatformInput] = useState('google_colab');

  // 1. FETCH INVENTORY
  const { data: identities, isLoading } = useQuery({
    queryKey: ['identities'],
    queryFn: async () => {
      const res = await apiClient.get<Identity[]>('/admin/identities');
      return res.data;
    },
    refetchInterval: 5000,
  });

  // 2. UPLOAD MUTATION
  const uploadMutation = useMutation({
    mutationFn: async () => {
      const cookies = JSON.parse(jsonInput);
      await apiClient.post('/admin/identities', {
        platform: platformInput,
        email: emailInput,
        cookies: cookies,
        user_agent: navigator.userAgent // O uno custom
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['identities'] });
      setJsonInput('');
      setEmailInput('');
      alert('Identidad Asegurada en Bóveda.');
    },
    onError: (e) => alert(`Error: ${e}`)
  });

  return (
    <div className="space-y-8">

      {/* --- SECTION A: INJECTION --- */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 bg-[#0a0a0a] border border-slate-800 rounded-xl p-6">
          <h2 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
            <Shield className="w-5 h-5 text-emerald-500" /> NEW IDENTITY INJECTION
          </h2>

          <div className="grid grid-cols-2 gap-4 mb-4">
             <input
               type="text"
               placeholder="Platform (e.g., google_colab)"
               value={platformInput}
               onChange={e => setPlatformInput(e.target.value)}
               className="bg-slate-900 border border-slate-700 p-2 rounded text-xs text-white"
             />
             <input
               type="email"
               placeholder="Associated Email (admin@gmail.com)"
               value={emailInput}
               onChange={e => setEmailInput(e.target.value)}
               className="bg-slate-900 border border-slate-700 p-2 rounded text-xs text-white"
             />
          </div>

          <textarea
            value={jsonInput}
            onChange={(e) => setJsonInput(e.target.value)}
            placeholder='Paste cookies.json content here...'
            className="w-full h-32 bg-slate-950 border border-slate-800 rounded p-4 font-mono text-[10px] text-emerald-500 mb-4"
          />

          <button
            onClick={() => uploadMutation.mutate()}
            disabled={uploadMutation.isPending || !jsonInput || !emailInput}
            className="bg-emerald-600 hover:bg-emerald-500 text-black px-6 py-2 rounded font-bold text-sm uppercase w-full flex justify-center items-center gap-2"
          >
            {uploadMutation.isPending ? <RefreshCw className="animate-spin w-4 h-4"/> : <Save className="w-4 h-4" />}
            SECURE CREDENTIALS
          </button>
        </div>

        {/* --- SECTION B: STATS SUMMARY --- */}
        <div className="bg-[#0a0a0a] border border-slate-800 rounded-xl p-6 flex flex-col justify-center items-center text-center">
            <div className="text-4xl font-black text-white mb-2">{identities?.length || 0}</div>
            <div className="text-xs text-slate-500 uppercase tracking-widest">Active Personas</div>
        </div>
      </div>

      {/* --- SECTION C: INVENTORY TABLE --- */}
      <div className="bg-[#0a0a0a] border border-slate-800 rounded-xl overflow-hidden">
        <table className="w-full text-left text-sm text-slate-400">
          <thead className="bg-slate-900/50 text-xs uppercase font-medium text-slate-500">
            <tr>
              <th className="px-6 py-4">Identity / Email</th>
              <th className="px-6 py-4">Platform</th>
              <th className="px-6 py-4">Usage Stats</th>
              <th className="px-6 py-4">Last Active</th>
              <th className="px-6 py-4 text-right">Status</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-800">
            {identities?.map((id) => (
              <tr key={id.id} className="hover:bg-slate-900/20">
                <td className="px-6 py-4 flex items-center gap-3">
                   <div className="p-2 bg-slate-800 rounded-full"><User className="w-4 h-4 text-slate-300"/></div>
                   <span className="font-mono text-white">{id.email}</span>
                </td>
                <td className="px-6 py-4">
                   <div className="flex items-center gap-2">
                     <Server className="w-3 h-3"/> {id.platform}
                   </div>
                </td>
                <td className="px-6 py-4 font-mono text-xs">
                   <span className="text-emerald-400">{id.usage_count}</span> missions
                </td>
                <td className="px-6 py-4 flex items-center gap-2 text-xs">
                   <Clock className="w-3 h-3"/>
                   {id.last_used_at ? new Date(id.last_used_at).toLocaleString() : 'Never'}
                </td>
                <td className="px-6 py-4 text-right">
                   <span className={`px-2 py-1 rounded text-[10px] uppercase font-bold border ${
                     id.status === 'active' ? 'bg-emerald-950/30 text-emerald-500 border-emerald-900' : 'bg-red-950/30 text-red-500 border-red-900'
                   }`}>
                     {id.status}
                   </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

    </div>
  );
}
