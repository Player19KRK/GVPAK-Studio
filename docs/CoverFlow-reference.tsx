import React, { useRef, useEffect } from 'react';
import { ChevronLeft, ChevronRight, Check } from './icons';

import './coverflow.css';
export interface CoverFlowAsset { id: string; name: string; previewUrl: string }
export interface CoverFlowProps {
 assets: CoverFlowAsset[]; activeAssetId: string | null;
 selectedAssetIds: string[]; projectName: string;
 onActiveAssetChange: (id: string) => void;
 onToggleSelection: (id: string) => void;
 onOpenProjectLibrary: () => void;
 keyboardEnabled?: boolean;
}
export const CoverFlow: React.FC<CoverFlowProps> = ({assets, activeAssetId, selectedAssetIds, projectName, onActiveAssetChange: selectAsset, onToggleSelection: toggleSelect, onOpenProjectLibrary, keyboardEnabled=true}) => {
  const isSelected = (id: string) => selectedAssetIds.includes(id);

  const containerRef = useRef<HTMLDivElement>(null);
  const currentIndex = Math.max(0, assets.findIndex((a) => a.id === activeAssetId));

  const handlePrev = () => {
    if (currentIndex > 0) {
      selectAsset(assets[currentIndex - 1].id);
    }
  };

  const handleNext = () => {
    if (currentIndex < assets.length - 1) {
      selectAsset(assets[currentIndex + 1].id);
    }
  };

  // 滚轮在 Cover Flow 区域左右翻页 (带轻量阈值防抖)
  const wheelAccumulator = useRef(0);
  const handleWheel = (e: React.WheelEvent) => {
    e.preventDefault();
    wheelAccumulator.current += e.deltaY || e.deltaX;
    if (wheelAccumulator.current > 35) {
      handleNext();
      wheelAccumulator.current = 0;
    } else if (wheelAccumulator.current < -35) {
      handlePrev();
      wheelAccumulator.current = 0;
    }
  };

  // 键盘左右箭头翻页快捷键
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
      if (e.key === 'ArrowLeft') handlePrev();
      if (e.key === 'ArrowRight') handleNext();
    };
    if (!keyboardEnabled) return;
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [currentIndex, assets, keyboardEnabled, selectAsset]);


  return (
    <div
      ref={containerRef}
      onWheel={handleWheel}
      className="cf-root"
    >
      {/* 顶部标题行: 项目名 (点击整体翻转进 XMB 项目库 REQ-057) */}
      <div className="cf-header">
        <button
          onClick={() => onOpenProjectLibrary()}
          className="cf-project"
          title="点击整体翻转进入 XMB 风格项目库视图"
        >
          <span className="cf-project-name">{projectName}</span>
          <span className="cf-hint">[翻转至项目库]</span>
        </button>

        <span className="cf-count">
          {assets.length} ITEMS IN PROJECT
        </span>
      </div>

      {/* 经典对称 3D 透视 Cover Flow 舞台 (选中项始终居中，两侧对称展开) */}
      <div className="cf-stage">
        {/* 左右翻页箭头按钮 */}
        <button
          onClick={handlePrev}
          disabled={currentIndex <= 0}
          className="cf-arrow cf-prev"
          title="上一张 (方向键左)"
        >
          <ChevronLeft className="cf-icon" />
        </button>
        <button
          onClick={handleNext}
          disabled={currentIndex >= assets.length - 1}
          className="cf-arrow cf-next"
          title="下一张 (方向键右)"
        >
          <ChevronRight className="cf-icon" />
        </button>

        {/* 居中对称 3D 卡片栈 */}
        <div className="cf-stack">
          {assets.map((asset, actualIndex) => {
            const offset = actualIndex - currentIndex;
            const isCenter = offset === 0;
            const selected = isSelected(asset.id);

            // 经典 Cover Flow 空间对称排列数学公式:
            // 1. 中心卡片 (offset === 0): translateX(0), rotateY(0), 居中面对用户
            // 2. 左侧卡片 (offset < 0): 向左推开并向右倾斜 (rotateY 45deg)
            // 3. 右侧卡片 (offset > 0): 向右推开并向左倾斜 (rotateY -45deg)
            let translateX = 0;
            let rotateY = 0;
            let translateZ = 0;
            let scale = 1;
            let opacity = 1;

            if (isCenter) {
              translateX = 0;
              rotateY = 0;
              translateZ = 60;
              scale = 1.06;
              opacity = 1;
            } else if (offset < 0) {
              const abs = Math.abs(offset);
              translateX = -(130 + (abs - 1) * 75);
              rotateY = 44;
              translateZ = -abs * 40;
              scale = Math.max(0.68, 0.95 - abs * 0.06);
              opacity = Math.max(0.25, 1 - abs * 0.18);
            } else {
              translateX = 130 + (offset - 1) * 75;
              rotateY = -44;
              translateZ = -offset * 40;
              scale = Math.max(0.68, 0.95 - offset * 0.06);
              opacity = Math.max(0.25, 1 - offset * 0.18);
            }

            // 超出视口范围的隐藏以提升渲染性能
            if (Math.abs(offset) > 7) return null;

            return (
              <div
                key={asset.id}
                onClick={() => selectAsset(asset.id)}
                style={{
                  transform: `translateX(${translateX}px) translateZ(${translateZ}px) rotateY(${rotateY}deg) scale(${scale})`,
                  opacity,
                  zIndex: isCenter ? 50 : 40 - Math.abs(offset),
                  transition: 'transform 0.35s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.3s ease',
                }}
                className={`cf-card ${
                  isCenter ? 'cf-current' : 'cf-side'
                }`}
              >
                {/* 缩略图 */}
                <img
                  src={asset.previewUrl}
                  alt={asset.name}
                  onError={(e) => {
                    (e.target as HTMLImageElement).src =
                      'data:image/svg+xml;utf8,' +
                      encodeURIComponent(
                        `<svg xmlns="http://www.w3.org/2000/svg" width="300" height="200" viewBox="0 0 300 200"><rect width="300" height="200" fill="#E2DFD8"/><text x="150" y="100" font-family="sans-serif" font-size="12" fill="#6B6862" text-anchor="middle">OFFLINE DEMO</text></svg>`
                      );
                  }}
                  className="cf-image"
                />

                {/* 批量多选勾选框 (D-02: Cover Flow 共享多选) */}
                <div
                  onClick={(e) => {
                    e.stopPropagation();
                    toggleSelect(asset.id);
                  }}
                  className={`cf-check ${
                    selected
                      ? 'cf-checked'
                      : 'cf-unchecked'
                  }`}
                  title="多选勾选"
                >
                  {selected && <Check className="cf-check-icon" />}
                </div>

                {/* 悬停浮层 */}
                <div className="cf-caption">
                  {asset.name}
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* 底部居中序号指示: ◀ IMAGE 036 / 150 (SELECTED: 24) ▶ */}
      <div className="cf-footer">
        <button
          onClick={handlePrev}
          disabled={currentIndex <= 0}
          className="cf-step"
        >
          ◀
        </button>

        <div className="cf-index">
          IMAGE <span className="cf-active-number">{String(assets.length ? currentIndex + 1 : 0).padStart(3, '0')}</span> / {String(assets.length).padStart(3, '0')}
        </div>

        {selectedAssetIds.length > 0 && (
          <span className="cf-selection">
            SELECTED: {selectedAssetIds.length}
          </span>
        )}

        <button
          onClick={handleNext}
          disabled={currentIndex >= assets.length - 1}
          className="cf-step"
        >
          ▶
        </button>
      </div>
    </div>
  );
};
