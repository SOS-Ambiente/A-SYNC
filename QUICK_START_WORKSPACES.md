# 🚀 Guia Rápido - Workspaces Colaborativos

## Início Rápido em 5 Minutos

### 1️⃣ Criar seu Primeiro Workspace

```typescript
import { useWorkspaceStore } from '@/stores/workspaceStore'

const workspaceStore = useWorkspaceStore()

// Criar workspace
const workspaceId = await workspaceStore.createWorkspace(
  'Meu Primeiro Workspace',
  'Espaço para colaboração em equipe'
)
```

### 2️⃣ Convidar Colaboradores

```typescript
// Convidar como Editor
await workspaceStore.inviteMember(
  workspaceId,
  'colaborador@email.com',
  'Editor'
)

// Convidar como Admin
await workspaceStore.inviteMember(
  workspaceId,
  'admin@email.com',
  'Admin'
)
```

### 3️⃣ Criar Pastas Compartilhadas

```typescript
// Criar pasta de documentos
const docsFolder = await workspaceStore.createSharedFolder(
  workspaceId,
  'Documentos',
  '/documentos'
)

// Criar pasta de código
const codeFolder = await workspaceStore.createSharedFolder(
  workspaceId,
  'Código Fonte',
  '/src'
)
```

### 4️⃣ Compartilhar Pastas com Membros

```typescript
// Dar acesso de editor à pasta de documentos
await workspaceStore.shareFolderWithMember(
  workspaceId,
  docsFolder,
  memberId,
  'Editor'
)
```

### 5️⃣ Upload de Arquivos

```typescript
import { useFilesStore } from '@/stores/filesStore'

const filesStore = useFilesStore()

// Upload com progresso
await filesStore.uploadFile(
  '/documentos/relatorio.pdf',
  (progress) => {
    console.log(`Upload: ${progress.progress}%`)
  }
)
```

## 🎯 Casos de Uso Comuns

### Projeto de Desenvolvimento

```typescript
// 1. Criar workspace do projeto
const projectId = await workspaceStore.createWorkspace(
  'Projeto Alpha',
  'Desenvolvimento do sistema Alpha'
)

// 2. Estrutura de pastas
const folders = {
  docs: await workspaceStore.createSharedFolder(projectId, 'Documentação', '/docs'),
  src: await workspaceStore.createSharedFolder(projectId, 'Código', '/src'),
  tests: await workspaceStore.createSharedFolder(projectId, 'Testes', '/tests'),
  assets: await workspaceStore.createSharedFolder(projectId, 'Assets', '/assets')
}

// 3. Convidar equipe
const team = [
  { email: 'dev1@team.com', role: 'Editor' },
  { email: 'dev2@team.com', role: 'Editor' },
  { email: 'lead@team.com', role: 'Admin' },
  { email: 'manager@team.com', role: 'Viewer' }
]

for (const member of team) {
  await workspaceStore.inviteMember(projectId, member.email, member.role)
}
```

### Compartilhamento de Documentos

```typescript
// 1. Workspace de documentos
const docsWorkspace = await workspaceStore.createWorkspace(
  'Documentos Corporativos',
  'Documentação e políticas da empresa'
)

// 2. Pastas por departamento
const departments = ['RH', 'Financeiro', 'TI', 'Marketing']

for (const dept of departments) {
  const folderId = await workspaceStore.createSharedFolder(
    docsWorkspace,
    dept,
    `/${dept.toLowerCase()}`
  )
  
  // Compartilhar apenas com membros do departamento
  // (implementar lógica de filtro)
}
```

### Projeto Acadêmico

```typescript
// 1. Workspace do grupo
const groupId = await workspaceStore.createWorkspace(
  'TCC - Grupo 5',
  'Trabalho de Conclusão de Curso'
)

// 2. Estrutura acadêmica
await workspaceStore.createSharedFolder(groupId, 'Artigo', '/artigo')
await workspaceStore.createSharedFolder(groupId, 'Apresentação', '/slides')
await workspaceStore.createSharedFolder(groupId, 'Código', '/code')
await workspaceStore.createSharedFolder(groupId, 'Referências', '/refs')

// 3. Convidar membros do grupo
const students = [
  'aluno1@universidade.edu',
  'aluno2@universidade.edu',
  'aluno3@universidade.edu'
]

for (const student of students) {
  await workspaceStore.inviteMember(groupId, student, 'Editor')
}

// 4. Convidar orientador como viewer
await workspaceStore.inviteMember(
  groupId,
  'orientador@universidade.edu',
  'Viewer'
)
```

## 🔐 Boas Práticas de Segurança

### Permissões

```typescript
// ✅ BOM: Princípio do menor privilégio
await workspaceStore.inviteMember(workspaceId, 'user@email.com', 'Viewer')

// ❌ EVITAR: Dar Admin sem necessidade
await workspaceStore.inviteMember(workspaceId, 'user@email.com', 'Admin')
```

### Organização

```typescript
// ✅ BOM: Estrutura clara de pastas
const structure = {
  '/public': 'Arquivos públicos',
  '/internal': 'Uso interno',
  '/confidential': 'Confidencial'
}

// ✅ BOM: Compartilhar pastas específicas
await workspaceStore.shareFolderWithMember(
  workspaceId,
  publicFolderId,
  memberId,
  'Viewer'
)
```

## 📊 Monitoramento

### Verificar Status

```typescript
// Carregar workspaces
await workspaceStore.loadWorkspaces()

// Verificar workspace atual
const current = workspaceStore.currentWorkspace
if (current) {
  console.log(`Workspace: ${current.name}`)
  console.log(`Membros: ${current.members.length}`)
  console.log(`Pastas: ${current.shared_folders.length}`)
}
```

### Estatísticas de Armazenamento

```typescript
// Obter estatísticas (implementar comando)
const stats = await invoke('get_distributed_storage_stats')

console.log(`Arquivos: ${stats.total_files}`)
console.log(`Hosts online: ${stats.online_hosts}/${stats.total_hosts}`)
console.log(`Capacidade: ${formatBytes(stats.network_capacity)}`)
```

## 🐛 Troubleshooting

### Convite não recebido

```typescript
// Verificar convites pendentes
const workspace = await workspaceStore.get_workspace(workspaceId)
const pendingInvites = Object.values(workspace.invites)
  .filter(invite => !invite.accepted)

console.log('Convites pendentes:', pendingInvites)
```

### Erro de permissão

```typescript
try {
  await workspaceStore.inviteMember(workspaceId, email, 'Admin')
} catch (error) {
  if (error.includes('PermissionDenied')) {
    console.log('Você não tem permissão para convidar membros')
    console.log('Apenas Admins e Owners podem convidar')
  }
}
```

### Pasta não encontrada

```typescript
// Verificar se pasta existe
const workspace = await workspaceStore.get_workspace(workspaceId)
const folder = workspace.shared_folders.find(f => f.id === folderId)

if (!folder) {
  console.log('Pasta não encontrada')
} else {
  console.log(`Pasta: ${folder.name}`)
  console.log(`Arquivos: ${Object.keys(folder.files).length}`)
}
```

## 🎨 Componentes UI

### Usar WorkspacePanel

```vue
<template>
  <div>
    <WorkspacePanel />
  </div>
</template>

<script setup>
import WorkspacePanel from '@/components/WorkspacePanel.vue'
</script>
```

### Usar StorageStats

```vue
<template>
  <div>
    <StorageStats :stats="storageStats" />
  </div>
</template>

<script setup>
import { ref } from 'vue'
import StorageStats from '@/components/StorageStats.vue'

const storageStats = ref({
  total_files: 42,
  total_size: 1024 * 1024 * 150,
  replicated_files: 38,
  online_hosts: 5,
  total_hosts: 8,
  network_capacity: 1024 * 1024 * 1024 * 10,
  network_used: 1024 * 1024 * 500
})
</script>
```

### View Completa

```vue
<template>
  <CollaborationView />
</template>

<script setup>
import CollaborationView from '@/views/CollaborationView.vue'
</script>
```

## 📚 Recursos Adicionais

- [Documentação Completa](./WORKSPACE_COLLABORATION.md)
- [Arquitetura P2P](./P2P_ARCHITECTURE.md)
- [Segurança](./SECURITY.md)

## 💡 Dicas

1. **Organize por projeto**: Crie um workspace para cada projeto
2. **Use permissões granulares**: Compartilhe pastas específicas, não o workspace inteiro
3. **Monitore o armazenamento**: Verifique regularmente as estatísticas
4. **Convide apenas necessários**: Menos membros = mais segurança
5. **Documente a estrutura**: Mantenha um README em cada workspace

## 🎯 Próximos Passos

1. ✅ Criar seu primeiro workspace
2. ✅ Convidar colaboradores
3. ✅ Organizar em pastas
4. ✅ Fazer upload de arquivos
5. 🔄 Explorar funcionalidades avançadas
6. 🔄 Integrar com seu workflow
7. 🔄 Contribuir com melhorias

---

**Pronto para começar?** Execute o aplicativo e crie seu primeiro workspace! 🚀
