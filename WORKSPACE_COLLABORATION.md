# 🏢 Sistema de Workspaces Colaborativos

## Visão Geral

Sistema completo de workspaces colaborativos com compartilhamento P2P de arquivos, onde cada host age como servidor de seus próprios arquivos e pastas compartilhadas.

## 🎯 Funcionalidades Principais

### 1. **Workspaces**
- Espaços de trabalho isolados para organização de projetos
- Cada workspace tem seu próprio conjunto de membros e permissões
- Suporte para múltiplos workspaces por usuário

### 2. **Sistema de Convites por Email**
- Convide colaboradores por email
- Convites com prazo de validade (7 dias)
- Sistema de aceitação de convites

### 3. **Níveis de Permissão**
- **Owner**: Dono do workspace (controle total)
- **Admin**: Pode gerenciar membros e pastas
- **Editor**: Pode criar e editar arquivos
- **Viewer**: Apenas visualização

### 4. **Pastas Compartilhadas**
- Crie pastas dentro de workspaces
- Compartilhe pastas com membros específicos
- Permissões granulares por pasta
- Cada host armazena seus próprios arquivos

### 5. **Armazenamento P2P Distribuído**
- Cada host age como servidor de seus arquivos
- Replicação automática entre hosts
- Descoberta de hosts online
- Seleção inteligente de host (menor latência)

## 🏗️ Arquitetura

### Backend (Rust)

#### `workspace.rs`
```rust
// Estruturas principais
- Workspace: Espaço de trabalho colaborativo
- WorkspaceMember: Membro com permissões
- SharedFolder: Pasta compartilhada
- WorkspaceInvite: Convite pendente
- WorkspaceManager: Gerenciador de workspaces
```

#### `p2p_storage.rs`
```rust
// Armazenamento distribuído
- FileReplication: Informações de replicação
- HostInfo: Informações de host na rede
- P2PStorageManager: Gerenciador de armazenamento P2P
```

### Frontend (Vue 3 + TypeScript)

#### `workspaceStore.ts`
- Store Pinia para gerenciamento de estado
- Comunicação com backend via Tauri

#### `WorkspacePanel.vue`
- Interface completa de workspaces
- Criação e gerenciamento
- Convites e permissões

#### `StorageStats.vue`
- Visualização de estatísticas
- Capacidade da rede
- Hosts online

## 📋 Comandos Tauri

### Workspaces
```typescript
// Listar workspaces do usuário
list_workspaces() -> Workspace[]

// Criar novo workspace
create_workspace(name: string, description: string) -> string

// Convidar membro
invite_workspace_member(
  workspace_id: string,
  email: string,
  permission: 'Viewer' | 'Editor' | 'Admin'
) -> string

// Aceitar convite
accept_workspace_invite(
  workspace_id: string,
  invite_id: string
) -> void

// Criar pasta compartilhada
create_shared_folder(
  workspace_id: string,
  name: string,
  path: string
) -> string

// Compartilhar pasta com membro
share_folder_with_member(
  workspace_id: string,
  folder_id: string,
  member_id: string,
  permission: string
) -> void
```

## 🚀 Como Usar

### 1. Criar Workspace
```typescript
const workspaceStore = useWorkspaceStore()

await workspaceStore.createWorkspace(
  'Meu Projeto',
  'Projeto colaborativo de desenvolvimento'
)
```

### 2. Convidar Membros
```typescript
await workspaceStore.inviteMember(
  workspaceId,
  'colaborador@email.com',
  'Editor'
)
```

### 3. Criar Pasta Compartilhada
```typescript
await workspaceStore.createSharedFolder(
  workspaceId,
  'Documentos',
  '/documentos'
)
```

### 4. Compartilhar Pasta
```typescript
await workspaceStore.shareFolderWithMember(
  workspaceId,
  folderId,
  memberId,
  'Editor'
)
```

## 🔐 Segurança

### Criptografia
- Todos os arquivos são criptografados com quantum-resistant encryption
- Chaves únicas por arquivo
- Blockchain para integridade

### Permissões
- Verificação de permissões em todas as operações
- Apenas donos e admins podem convidar
- Permissões granulares por pasta

### Convites
- Convites expiram em 7 dias
- Verificação de email ao aceitar
- Apenas um uso por convite

## 🌐 Rede P2P

### Descoberta de Hosts
- mDNS para rede local
- DHT para descoberta global
- Registro automático de hosts

### Replicação
- Replicação automática entre hosts
- Fator de replicação configurável
- Verificação de integridade

### Seleção de Host
- Ordenação por latência
- Preferência por hosts online
- Fallback para hosts alternativos

## 📊 Estatísticas

### Por Workspace
- Total de membros
- Total de pastas compartilhadas
- Total de arquivos

### Rede Distribuída
- Hosts online/total
- Capacidade da rede
- Espaço usado
- Taxa de replicação

## 🔄 Sincronização

### Upload
1. Arquivo é criptografado localmente
2. Registrado no workspace/pasta
3. Distribuído para hosts na rede
4. Replicado conforme configuração

### Download
1. Busca hosts que possuem o arquivo
2. Seleciona melhor host (latência)
3. Download e verificação
4. Descriptografia local

## 🎨 Interface

### Painel de Workspaces
- Lista de workspaces
- Criação de novos workspaces
- Seleção de workspace ativo

### Detalhes do Workspace
- Lista de membros com permissões
- Pastas compartilhadas
- Botões de ação

### Dialogs
- Criar workspace
- Convidar membro
- Criar pasta compartilhada

## 🛠️ Melhorias Futuras

### Funcionalidades
- [ ] Chat em tempo real por workspace
- [ ] Notificações de atividades
- [ ] Histórico de alterações
- [ ] Versionamento de arquivos
- [ ] Sincronização offline
- [ ] Conflitos de edição simultânea

### Performance
- [ ] Cache de metadados
- [ ] Pré-carregamento de arquivos
- [ ] Compressão adaptativa
- [ ] Delta sync

### Segurança
- [ ] 2FA para workspaces sensíveis
- [ ] Auditoria de acessos
- [ ] Criptografia end-to-end para chat
- [ ] Revogação de acesso

## 📝 Exemplo Completo

```typescript
// 1. Criar workspace
const wsId = await workspaceStore.createWorkspace(
  'Projeto Alpha',
  'Desenvolvimento colaborativo'
)

// 2. Convidar equipe
await workspaceStore.inviteMember(wsId, 'dev1@team.com', 'Editor')
await workspaceStore.inviteMember(wsId, 'dev2@team.com', 'Editor')
await workspaceStore.inviteMember(wsId, 'manager@team.com', 'Admin')

// 3. Criar estrutura de pastas
const docsId = await workspaceStore.createSharedFolder(
  wsId, 'Documentos', '/docs'
)
const codeId = await workspaceStore.createSharedFolder(
  wsId, 'Código', '/code'
)

// 4. Compartilhar pastas
await workspaceStore.shareFolderWithMember(
  wsId, docsId, dev1Id, 'Editor'
)
await workspaceStore.shareFolderWithMember(
  wsId, codeId, dev2Id, 'Editor'
)

// 5. Upload de arquivos
await filesStore.uploadFile('/docs/README.md')
await filesStore.uploadFile('/code/main.rs')
```

## 🎯 Benefícios

### Para Usuários
- ✅ Colaboração fácil e intuitiva
- ✅ Controle granular de permissões
- ✅ Sem necessidade de servidor central
- ✅ Privacidade e segurança

### Para Desenvolvedores
- ✅ Arquitetura modular
- ✅ Fácil extensão
- ✅ Bem documentado
- ✅ Type-safe (Rust + TypeScript)

### Para a Rede
- ✅ Descentralização total
- ✅ Escalabilidade horizontal
- ✅ Resistência a falhas
- ✅ Eficiência de armazenamento
