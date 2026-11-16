# 소설도우미 (Novel Studio)

Offline-first 소설 전문 에디터 + Multi-Provider AI 집필 보조 + 작품 설정 관리(Context Engine) 데스크톱 애플리케이션입니다.

## 핵심 방향
- **Offline-first**: 인터넷 없이도 집필, 편집, 검색(SQLite FTS5), 버전 관리, 설정 관리 가능
- **Multi-Provider AI**: OpenAI(Chat Completions / Responses), Anthropic(Messages), Google(Gemini Interactions / generateContent), Ollama, OpenAI-Compatible API 지원
- **Non-destructive Editing**: AI 수정 제안을 원고 `revision` 검증 후 비교·선택하여 반영
- **Bilingual**: 한국어 / 영어 UI(`i18next`) 및 작품별 집필 언어 분리 관리

## 기술 스택
- **Desktop**: Tauri 2.x (Windows 우선, macOS 확장)
- **Frontend**: React + TypeScript + Vite + Tailwind CSS + shadcn/ui + Tiptap 3 + Zustand
- **Backend**: Rust + SQLite (FTS5 `trigram`) + `reqwest` + OS Credential Store

## 개발 단계
1. **Phase 1**: 소설 에디터 코어 (Tauri 2.x + Tiptap 3 + SQLite + KO/EN i18n + 버전 관리)
2. **Phase 2**: Multi-Provider AI 어댑터 & 키 보안 저장소
3. **Phase 3**: AI 집필 보조 & 비파괴적 수정 제안(`ai_proposals`)
4. **Phase 4**: Context Engine & 설정 충돌/복선 분석
5. **Phase 5**: DOCX/EPUB/PDF 내보내기, 백업 및 배포
