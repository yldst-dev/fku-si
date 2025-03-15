# FKU-SI (Tracking Parameter Remover)

YouTube, YouTube Music, Spotify 공유 링크에서 추적 파라미터(si)를 제거해주는 텔레그램 봇입니다.

## 기능

- YouTube, YouTube Music, Spotify 링크에서 추적 파라미터(si)를 자동으로 제거
- 관리자 권한이 있을 경우: 원본 메시지 삭제 후 정리된 링크로 대체
- 관리자 권한이 없을 경우: 인라인 버튼으로 정리된 링크 제공

## 설치 및 실행

### 옵션 1: 직접 실행

1. Rust와 Cargo가 설치되어 있어야 합니다.
2. 환경 변수 파일을 설정합니다:
   - `env.example` 파일을 `.env`로 복사합니다.
   - `.env` 파일에 텔레그램 봇 토큰을 입력합니다.
   - 필요에 따라 로깅 레벨을 설정합니다(`error`, `warn`, `info`, `debug`, `trace`).
   ```bash
   cp .env.example .env
   # .env 파일을 편집하여 TELOXIDE_TOKEN에 봇 토큰을 입력하세요
   ```

3. 프로젝트를 빌드하고 실행합니다:
   ```bash
   cargo build --release
   ./target/release/fku-si
   ```

### 옵션 2: Docker로 실행

1. Docker와 Docker Compose가 설치되어 있어야 합니다.
2. 환경 변수 파일을 설정합니다:
   ```bash
   cp .env.example .env
   # .env 파일을 편집하여 TELOXIDE_TOKEN에 봇 토큰을 입력하세요
   ```

3. Docker Compose를 사용하여 빌드하고 실행합니다:
   ```bash
   # 빌드 및 실행
   docker-compose up -d
   
   # 로그 확인
   docker-compose logs -f
   
   # 중지
   docker-compose down
   ```

4. 또는 Docker 명령어로 직접 빌드하고 실행할 수 있습니다:
   ```bash
   # 이미지 빌드
   docker build -t fku-si .
   
   # 컨테이너 실행
   docker run -d --name fku-si -v $(pwd)/.env:/app/.env:ro --restart unless-stopped fku-si
   
   # 로그 확인
   docker logs -f fku-si
   ```

## 로깅 레벨 설정

.env 파일에서 `RUST_LOG` 환경 변수로 로깅 레벨을 설정할 수 있습니다:

- `error`: 오류 메시지만 표시
- `warn`: 경고 및 오류 메시지 표시
- `info`: 정보, 경고, 오류 메시지 표시 (기본값)
- `debug`: 디버그, 정보, 경고, 오류 메시지 표시
- `trace`: 모든 로그 메시지 표시

예시:
```
RUST_LOG=debug  # 디버그 수준 로깅 활성화
```

## 봇 명령어

- `/help` - 사용 가능한 명령어 목록과 봇의 동작 방식을 표시합니다
  - 그룹에서 봇에 관리자 권한이 있을 때: 원본 메시지를 삭제하고 사용자 닉네임과 함께 정리된 링크를 새 메시지로 전송
  - 그룹에서 봇에 관리자 권한이 없을 때: 원본 메시지에 답장으로 인라인 버튼 형태의 정리된 링크 제공
- `/about` - 봇에 대한 정보 표시
- `/test [URL]` - URL에서 si 파라미터 제거 테스트

## 사용 방법

1. 봇을 그룹 채팅에 초대합니다.
2. 관리자 권한을 부여하면 자동으로 si 파라미터가 포함된 메시지를 감지하고 처리합니다.
3. 개인 대화에서도 사용 가능합니다.

## 예시

### 변환 예시:

- 변환 전: `https://music.youtube.com/watch?v=nmYDYalgb5w&si=GGi18ac_fxnx4F1b`
- 변환 후: `https://music.youtube.com/watch?v=nmYDYalgb5w`

- 변환 전: `https://open.spotify.com/track/1FYWnRofuIgJf62AnX8i5S?si=bf00147df50f4141`
- 변환 후: `https://open.spotify.com/track/1FYWnRofuIgJf62AnX8i5S`

- 변환 전: `https://youtu.be/Vc-ByDGOuQE?si=qIy-ihfrRKmDAPZP`
- 변환 후: `https://youtu.be/Vc-ByDGOuQE` 
