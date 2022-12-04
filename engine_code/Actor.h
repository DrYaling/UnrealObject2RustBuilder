
enum EActorUpdateOverlapsMethod : uint8
{
	UseConfigDefault,
	AlwaysUpdate,
	OnlyUpdateMovable,
	NeverUpdate
};


class  AActor : public UObject
{
public:

	AActor();

	AActor(const FObjectInitializer& ObjectInitializer);
private:

	void InitializeDefaults();
public:

	void GetLifetimeReplicatedProps(TArray<FLifetimeProperty>& OutLifetimeProps) const override;


	uint8 bNetTemporary:1;

	uint8 bNetStartup:1;

	uint8 bOnlyRelevantToOwner:1;

	uint8 bAlwaysRelevant:1;    

	virtual void OnRep_ReplicateMovement();
private:

	uint8 bReplicateMovement:1;    

	uint8 bHidden:1;
	uint8 bTearOff:1;

	uint8 bForceNetAddressable:1;
public:

	bool GetTearOff() const;

	virtual void TearOff();

	uint8 bExchangedRoles:1;

	uint8 bNetLoadOnClient:1;

	uint8 bNetUseOwnerRelevancy:1;

	uint8 bRelevantForNetworkReplays:1;

	uint8 bRelevantForLevelBounds:1;

	uint8 bReplayRewindable:1;

	uint8 bAllowTickBeforeBeginPlay:1;
private:

	uint8 bAutoDestroyWhenFinished:1;

	uint8 bCanBeDamaged:1;
public:

	uint8 bBlockInput:1;

	uint8 bCollideWhenPlacing:1;

	uint8 bFindCameraComponentWhenViewTarget:1;

	uint8 bGenerateOverlapEventsDuringLevelStreaming:1;

	uint8 bIgnoresOriginShifting:1;

	uint8 bEnableAutoLODGeneration:1;

	uint8 bIsEditorOnlyActor:1;

	uint8 bActorSeamlessTraveled:1;

	virtual bool HasNetOwner() const;

	virtual bool HasLocalNetOwner() const;
bool GetAutoDestroyWhenFinished() const ;
	void SetAutoDestroyWhenFinished(bool bVal);
protected:

	uint8 bReplicates:1;

void SetRemoteRoleForBackwardsCompat(const ENetRole InRemoteRole) ;

	virtual void OnRep_Owner();

	uint8 bCanBeInCluster:1;

	uint8 bAllowReceiveTickEventOnDedicatedServer:1;

	uint8 bNetCheckedInitialPhysicsState : 1;
private:

	uint8 bHasFinishedSpawning:1;

	uint8 bActorInitialized:1;

	uint8 bActorBeginningPlayFromLevelStreaming:1;

	uint8 bTickFunctionsRegistered:1;

	uint8 bHasDeferredComponentRegistration:1;

	uint8 bRunningUserConstructionScript:1;

	uint8 bActorEnableCollision:1;

	uint8 bActorIsBeingDestroyed:1;

	uint8 bActorWantsDestroyDuringBeginPlay : 1;

	enum EActorBeginPlayState : uint8
	{
		HasNotBegunPlay,
		BeginningPlay,
		HasBegunPlay,
	};

	EActorBeginPlayState ActorHasBegunPlay:2;
	static uint32 BeginPlayCallDepth;
protected:

	EActorUpdateOverlapsMethod UpdateOverlapsMethodDuringLevelStreaming;
public:

	EActorUpdateOverlapsMethod GetUpdateOverlapsMethodDuringLevelStreaming() const;
private:

	EActorUpdateOverlapsMethod DefaultUpdateOverlapsMethodDuringLevelStreaming;

	void UpdateInitialOverlaps(bool bFromLevelStreaming);

	TEnumAsByte<enum ENetRole> RemoteRole;	
public:

	void SetReplicates(bool bInReplicates);

	virtual void SetReplicateMovement(bool bInReplicateMovement);

	void SetAutonomousProxy(const bool bInAutonomousProxy, const bool bAllowForcePropertyCompare=true);

	void CopyRemoteRoleFrom(const AActor* CopyFromActor);

ENetRole GetLocalRole() const ;

	ENetRole GetRemoteRole() const;

	void SetNetAddressable();
private:

public:

	float InitialLifeSpan;

	float CustomTimeDilation;

	float CreationTime;
protected:


	TObjectPtr<AActor> Owner;
protected:

	FName NetDriverName;
public:

const struct FRepAttachment& GetAttachmentReplication() const ;

	virtual void OnRep_AttachmentReplication();
private:

	TEnumAsByte<enum ENetRole> Role;
public:

	TEnumAsByte<enum ENetDormancy> NetDormancy;

	virtual bool IsReplicationPausedForConnection(const FNetViewer& ConnectionOwnerNetViewer);

	virtual void OnReplicationPausedChanged(bool bIsReplicationPaused);

	ESpawnActorCollisionHandlingMethod SpawnCollisionHandlingMethod;

	TEnumAsByte<EAutoReceiveInput::Type> AutoReceiveInput;

	int32 InputPriority;

	TObjectPtr<class UInputComponent> InputComponent;

	float NetCullDistanceSquared;   

	int32 NetTag;

	float NetUpdateFrequency;

	float MinNetUpdateFrequency;

	float NetPriority;
private:

	float LastRenderTime;
	friend struct FActorLastRenderTime;
public:

	void SetNetDriverName(FName NewNetDriverName);

FName GetNetDriverName() const ;

	virtual bool ReplicateSubobjects(class UActorChannel *Channel, class FOutBunch *Bunch, FReplicationFlags *RepFlags);

	virtual void OnSubobjectCreatedFromReplication(UObject *NewSubobject);

	virtual void OnSubobjectDestroyFromReplication(UObject *Subobject);

	virtual void PreReplication(IRepChangedPropertyTracker & ChangedPropertyTracker);

	virtual void PreReplicationForReplay(IRepChangedPropertyTracker & ChangedPropertyTracker);

	virtual void RewindForReplay();

	void CallPreReplication(UNetDriver* NetDriver);	
private:

	TObjectPtr<class APawn> Instigator;
public:

	virtual void OnRep_Instigator();

	TArray<TObjectPtr<AActor>> Children;
protected:

	TObjectPtr<USceneComponent> RootComponent;

	TArray<TObjectPtr<class AMatineeActor>> ControllingMatineeActors;

	FTimerHandle TimerHandle_LifeSpanExpired;
public:
private:
public:

bool AllowReceiveTickEventOnDedicatedServer() const ;

bool IsRunningUserConstructionScript() const ;

	TArray< FName > Layers;
private:

	TWeakObjectPtr<UChildActorComponent> ParentComponent;	
public:

	bool IsPackageExternal() const;

	uint8 bActorIsBeingConstructed : 1;
public:

	TArray<FName> Tags;

	FTakeAnyDamageSignature OnTakeAnyDamage;

	FTakePointDamageSignature OnTakePointDamage;

	FTakeRadialDamageSignature OnTakeRadialDamage;

	FActorBeginOverlapSignature OnActorBeginOverlap;

	FActorEndOverlapSignature OnActorEndOverlap;

	FActorBeginCursorOverSignature OnBeginCursorOver;

	FActorEndCursorOverSignature OnEndCursorOver;

	FActorOnClickedSignature OnClicked;

	FActorOnReleasedSignature OnReleased;

	FActorOnInputTouchBeginSignature OnInputTouchBegin;

	FActorOnInputTouchEndSignature OnInputTouchEnd;

	FActorBeginTouchOverSignature OnInputTouchEnter;

	FActorEndTouchOverSignature OnInputTouchLeave;

	FActorHitSignature OnActorHit;

	virtual void EnableInput(class APlayerController* PlayerController);

	virtual void DisableInput(class APlayerController* PlayerController);

	float GetInputAxisValue(const FName InputAxisName) const;

	float GetInputAxisKeyValue(const FKey InputAxisKey) const;

	FVector GetInputVectorAxisValue(const FKey InputAxisKey) const;

	APawn* GetInstigator() const;


	AController* GetInstigatorController() const;


	const FTransform& GetTransform() const;

	 const FTransform& ActorToWorld() const;

	FVector K2_GetActorLocation() const;

	bool K2_SetActorLocation(FVector NewLocation, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);

	FRotator K2_GetActorRotation() const;

	FVector GetActorForwardVector() const;

	FVector GetActorUpVector() const;

	FVector GetActorRightVector() const;

	virtual void GetActorBounds(bool bOnlyCollidingComponents, FVector& Origin, FVector& BoxExtent, bool bIncludeFromChildActors = false) const;

	USceneComponent* K2_GetRootComponent() const;

	virtual FVector GetVelocity() const;


	bool K2_SetActorRotation(FRotator NewRotation, bool bTeleportPhysics);


	bool K2_SetActorLocationAndRotation(FVector NewLocation, FRotator NewRotation, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);


	void SetActorScale3D(FVector NewScale3D);

	FVector GetActorScale3D() const;

	float GetDistanceTo(const AActor* OtherActor) const;

	float GetSquaredDistanceTo(const AActor* OtherActor) const;

	float GetHorizontalDistanceTo(const AActor* OtherActor) const;

	float GetSquaredHorizontalDistanceTo(const AActor* OtherActor) const;

	float GetVerticalDistanceTo(const AActor* OtherActor) const;

	float GetDotProductTo(const AActor* OtherActor) const;

	float GetHorizontalDotProductTo(const AActor* OtherActor) const;

	void K2_AddActorWorldOffset(FVector DeltaLocation, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);

	void K2_AddActorWorldRotation(FRotator DeltaRotation, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);

	void K2_AddActorWorldTransform(const FTransform& DeltaTransform, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);

	void K2_AddActorWorldTransformKeepScale(const FTransform& DeltaTransform, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);

	bool K2_SetActorTransform(const FTransform& NewTransform, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);

	void K2_AddActorLocalOffset(FVector DeltaLocation, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);

	void K2_AddActorLocalRotation(FRotator DeltaRotation, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);

	void K2_AddActorLocalTransform(const FTransform& NewTransform, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);

	void K2_SetActorRelativeLocation(FVector NewRelativeLocation, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);

	void K2_SetActorRelativeRotation(FRotator NewRelativeRotation, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);

	void K2_SetActorRelativeTransform(const FTransform& NewRelativeTransform, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);

	void SetActorRelativeScale3D(FVector NewRelativeScale);

	FVector GetActorRelativeScale3D() const;

	virtual void SetActorHiddenInGame(bool bNewHidden);

	void SetActorEnableCollision(bool bNewActorEnableCollision);

	bool GetActorEnableCollision() const;

	virtual void K2_DestroyActor();

	bool HasAuthority() const;

	UActorComponent* AddComponent(FName TemplateName, bool bManualAttachment, const FTransform& RelativeTransform, const UObject* ComponentTemplateContext, bool bDeferredFinish = false);

	UActorComponent* AddComponentByClass(TSubclassOf<UActorComponent> Class, bool bManualAttachment, const FTransform& RelativeTransform, bool bDeferredFinish);

	void FinishAddComponent(UActorComponent* Component, bool bManualAttachment, const FTransform& RelativeTransform);


	void K2_AttachToComponent(USceneComponent* Parent, FName SocketName, EAttachmentRule LocationRule, EAttachmentRule RotationRule, EAttachmentRule ScaleRule, bool bWeldSimulatedBodies);

	void AttachToComponent(USceneComponent* Parent, const FAttachmentTransformRules& AttachmentRules, FName SocketName = NAME_None);


	void AttachToActor(AActor* ParentActor, const FAttachmentTransformRules& AttachmentRules, FName SocketName = NAME_None);

	void K2_AttachToActor(AActor* ParentActor, FName SocketName, EAttachmentRule LocationRule, EAttachmentRule RotationRule, EAttachmentRule ScaleRule, bool bWeldSimulatedBodies);



	void DetachFromActor(const FDetachmentTransformRules& DetachmentRules);

	void DetachAllSceneComponents(class USceneComponent* InParentComponent, const FDetachmentTransformRules& DetachmentRules);

	bool ActorHasTag(FName Tag) const;

	float GetActorTimeDilation() const;

	float GetActorTimeDilation(const UWorld& ActorWorld) const;

	virtual void AddTickPrerequisiteActor(AActor* PrerequisiteActor);

	virtual void AddTickPrerequisiteComponent(UActorComponent* PrerequisiteComponent);

	virtual void RemoveTickPrerequisiteActor(AActor* PrerequisiteActor);

	virtual void RemoveTickPrerequisiteComponent(UActorComponent* PrerequisiteComponent);

	bool GetTickableWhenPaused();

	void SetTickableWhenPaused(bool bTickableWhenPaused);

	float GetGameTimeSinceCreation() const;
protected:

	void ReceiveBeginPlay();

	virtual void BeginPlay();


public:

	void DispatchBeginPlay(bool bFromLevelStreaming = false);

bool IsActorInitialized() const ;

bool IsActorBeginningPlay() const ;

bool HasActorBegunPlay() const ;

bool IsActorBeginningPlayFromLevelStreaming() const ;

	bool IsActorBeingDestroyed() const ;

	void ReceiveAnyDamage(float Damage, const class UDamageType* DamageType, class AController* InstigatedBy, AActor* DamageCauser);

	void ReceiveRadialDamage(float DamageReceived, const class UDamageType* DamageType, FVector Origin, const struct FHitResult& HitInfo, class AController* InstigatedBy, AActor* DamageCauser);

	void ReceivePointDamage(float Damage, const class UDamageType* DamageType, FVector HitLocation, FVector HitNormal, class UPrimitiveComponent* HitComponent, FName BoneName, FVector ShotFromDirection, class AController* InstigatedBy, AActor* DamageCauser, const FHitResult& HitInfo);

	void ReceiveTick(float DeltaSeconds);

	virtual void NotifyActorBeginOverlap(AActor* OtherActor);

	void ReceiveActorBeginOverlap(AActor* OtherActor);

	virtual void NotifyActorEndOverlap(AActor* OtherActor);

	void ReceiveActorEndOverlap(AActor* OtherActor);

	virtual void NotifyActorBeginCursorOver();

	void ReceiveActorBeginCursorOver();

	virtual void NotifyActorEndCursorOver();

	void ReceiveActorEndCursorOver();













	void GetOverlappingActors(TArray<AActor*>& OverlappingActors, TSubclassOf<AActor> ClassFilter=nullptr) const;

	void GetOverlappingActors(TSet<AActor*>& OverlappingActors, TSubclassOf<AActor> ClassFilter=nullptr) const;

	void GetOverlappingComponents(TArray<UPrimitiveComponent*>& OverlappingComponents) const;

	void GetOverlappingComponents(TSet<UPrimitiveComponent*>& OverlappingComponents) const;

	virtual void NotifyHit(class UPrimitiveComponent* MyComp, AActor* Other, class UPrimitiveComponent* OtherComp, bool bSelfMoved, FVector HitLocation, FVector HitNormal, FVector NormalImpulse, const FHitResult& Hit);

	void ReceiveHit(class UPrimitiveComponent* MyComp, AActor* Other, class UPrimitiveComponent* OtherComp, bool bSelfMoved, FVector HitLocation, FVector HitNormal, FVector NormalImpulse, const FHitResult& Hit);

	virtual void SetLifeSpan( float InLifespan );

	virtual float GetLifeSpan() const;

	void UserConstructionScript();

	bool Destroy(bool bNetForce = false, bool bShouldModifyLevel = true );

	void ReceiveDestroyed();

	FActorDestroyedSignature OnDestroyed;

	FActorEndPlaySignature OnEndPlay;
	virtual bool CheckDefaultSubobjectsInternal() const override;
	virtual void PostInitProperties() override;
	virtual void ProcessEvent( UFunction* Function, void* Parameters ) override;
	virtual int32 GetFunctionCallspace( UFunction* Function, FFrame* Stack ) override;
	virtual bool CallRemoteFunction( UFunction* Function, void* Parameters, FOutParmRec* OutParms, FFrame* Stack ) override;
	virtual void Serialize(FArchive& Ar) override;
	virtual void PostLoad() override;
	virtual void PostLoadSubobjects( FObjectInstancingGraph* OuterInstanceGraph ) override;
	virtual void BeginDestroy() override;
	virtual bool IsReadyForFinishDestroy() override;
	virtual bool Rename( const TCHAR* NewName=nullptr, UObject* NewOuter=nullptr, ERenameFlags Flags=REN_None ) override;
	virtual void PostRename( UObject* OldOuter, const FName OldName ) override;
	virtual bool CanBeInCluster() const override;
	static void AddReferencedObjects(UObject* InThis, FReferenceCollector& Collector);
	virtual bool IsEditorOnly() const override;
	virtual bool IsAsset() const override;
	virtual bool PreSaveRoot(const TCHAR* InFilename) override;
	virtual void PostSaveRoot(bool bCleanupIsRequired) override;
	virtual void PreSave(const ITargetPlatform* TargetPlatform) override;

	bool IsMainPackageActor() const;
	static AActor* FindActorInPackage(UPackage* InPackage);

	virtual void GatherCurrentMovement();

	 bool IsOwnedBy( const AActor* TestOwner ) const;

USceneComponent* GetRootComponent() const ;

virtual USceneComponent* GetDefaultAttachComponent() const ;

	bool SetRootComponent(USceneComponent* NewRootComponent);

	 const FTransform& GetActorTransform() const;

	 FVector GetActorLocation() const;

	 FRotator GetActorRotation() const;

	 FVector GetActorScale() const;

	 FQuat GetActorQuat() const;

	virtual void ApplyWorldOffset(const FVector& InOffset, bool bWorldShift);

virtual bool IsLevelBoundsRelevant() const ;

	virtual bool IsHLODRelevant() const;

	void SetLODParent(class UPrimitiveComponent* InLODParent, float InParentDrawDistance);

	virtual float GetNetPriority(const FVector& ViewPos, const FVector& ViewDir, class AActor* Viewer, AActor* ViewTarget, UActorChannel* InChannel, float Time, bool bLowBandwidth);

	virtual float GetReplayPriority(const FVector& ViewPos, const FVector& ViewDir, class AActor* Viewer, AActor* ViewTarget, UActorChannel* const InChannel, float Time);

	virtual bool GetNetDormancy(const FVector& ViewPos, const FVector& ViewDir, class AActor* Viewer, AActor* ViewTarget, UActorChannel* InChannel, float Time, bool bLowBandwidth);

virtual void OnActorChannelOpen(class FInBunch& InBunch, class UNetConnection* Connection) ;

virtual bool UseShortConnectTimeout() const ;

virtual void OnSerializeNewActor(class FOutBunch& OutBunch) ;

virtual void OnNetCleanup(class UNetConnection* Connection) ;

	void ExchangeNetRoles(bool bRemoteOwner);

	void SwapRoles();

	void RegisterAllActorTickFunctions(bool bRegister, bool bDoComponents);

	void SetActorTickEnabled(bool bEnabled);

	bool IsActorTickEnabled() const;

	void SetActorTickInterval(float TickInterval);

	float GetActorTickInterval() const;

	virtual void TickActor( float DeltaTime, enum ELevelTick TickType, FActorTickFunction& ThisTickFunction );

	virtual void PostActorCreated();

	virtual void LifeSpanExpired();

	virtual void PreNetReceive() override;

	virtual void PostNetReceive() override;

	virtual void PostNetReceiveRole();

	virtual bool IsNameStableForNetworking() const override;

	virtual bool IsSupportedForNetworking() const override;

	virtual void GetSubobjectsWithStableNamesForNetworking(TArray<UObject*> &ObjList) override;

	virtual void PostNetInit();

	virtual void OnRep_ReplicatedMovement();

	virtual void PostNetReceiveLocationAndRotation();

	virtual void PostNetReceiveVelocity(const FVector& NewVelocity);

	virtual void PostNetReceivePhysicState();
protected:

	void SyncReplicatedPhysicsSimulation();
public:

	virtual void SetOwner( AActor* NewOwner );

	AActor* GetOwner() const;


	virtual bool CheckStillInWorld();

	void ClearComponentOverlaps();

	void UpdateOverlaps(bool bDoNotifies=true);

	bool IsOverlappingActor(const AActor* Other) const;

	bool IsMatineeControlled() const;

	bool IsRootComponentStatic() const;

	bool IsRootComponentStationary() const;

	bool IsRootComponentMovable() const;

bool CanEverTick() const ;

	virtual void Tick( float DeltaSeconds );

	virtual bool ShouldTickIfViewportsOnly() const;
protected:

	bool IsWithinNetRelevancyDistance(const FVector& SrcLocation) const;
public:	

	virtual bool IsNetRelevantFor(const AActor* RealViewer, const AActor* ViewTarget, const FVector& SrcLocation) const;

	virtual bool IsReplayRelevantFor(const AActor* RealViewer, const AActor* ViewTarget, const FVector& SrcLocation, const float CullDistanceSquared) const;

	virtual bool IsRelevancyOwnerFor(const AActor* ReplicatedActor, const AActor* ActorOwner, const AActor* ConnectionActor) const;

	void PostSpawnInitialize(FTransform const& SpawnTransform, AActor* InOwner, APawn* InInstigator, bool bRemoteOwned, bool bNoFail, bool bDeferConstruction);

	void FinishSpawning(const FTransform& Transform, bool bIsDefaultTransform = false, const FComponentInstanceDataCache* InstanceDataCache = nullptr);

	void PostActorConstruction();
public:

	virtual void PreInitializeComponents();

	virtual void PostInitializeComponents();

	void AddControllingMatineeActor( AMatineeActor& InMatineeActor );

	void RemoveControllingMatineeActor( AMatineeActor& InMatineeActor );

	virtual void DispatchPhysicsCollisionHit(const struct FRigidBodyCollisionInfo& MyInfo, const struct FRigidBodyCollisionInfo& OtherInfo, const FCollisionImpactData& RigidCollisionData);

	virtual const AActor* GetNetOwner() const;

	virtual class UPlayer* GetNetOwningPlayer();

	virtual class UNetConnection* GetNetConnection() const;

	virtual bool DestroyNetworkActorHandled();

	ENetMode GetNetMode() const;

	bool IsNetMode(ENetMode Mode) const;


	void SetNetDormancy(ENetDormancy NewDormancy);

	void FlushNetDormancy();

	void ForcePropertyCompare();

	bool IsChildActor() const;

	virtual bool IsSelectionParentOfAttachedActors() const;

	virtual bool IsSelectionChild() const;

	virtual AActor* GetSelectionParent() const;

	virtual AActor* GetRootSelectionParent() const;

	virtual void PushSelectionToProxies();

	void GetAllChildActors(TArray<AActor*>& ChildActors, bool bIncludeDescendants = true) const;

	UChildActorComponent* GetParentComponent() const;

	AActor* GetParentActor() const;

	virtual void RegisterAllComponents();

	virtual void PreRegisterAllComponents();

	virtual void PostRegisterAllComponents();

bool HasDeferredComponentRegistration() const ;

	bool HasValidRootComponent();

	virtual void UnregisterAllComponents(bool bForReregister = false);

	virtual void PostUnregisterAllComponents();

	virtual void ReregisterAllComponents();

	bool IncrementalRegisterComponents(int32 NumComponentsToRegister, FRegisterComponentContext* Context = nullptr);

	void MarkComponentsRenderStateDirty();

	void UpdateComponentTransforms();

	void InitializeComponents();

	void UninitializeComponents();


	virtual void MarkComponentsAsPendingKill();

	 bool IsPendingKillPending() const;

	void InvalidateLightingCache();

	virtual void InvalidateLightingCacheDetailed(bool bTranslationOnly);

	virtual bool TeleportTo( const FVector& DestLocation, const FRotator& DestRotation, bool bIsATest=false, bool bNoCheck=false );

	bool K2_TeleportTo( FVector DestLocation, FRotator DestRotation );

	virtual void TeleportSucceeded(bool bIsATest) {}

	bool ActorLineTraceSingle(struct FHitResult& OutHit, const FVector& Start, const FVector& End, ECollisionChannel TraceChannel, const struct FCollisionQueryParams& Params) const;

	float ActorGetDistanceToCollision(const FVector& Point, ECollisionChannel TraceChannel, FVector& ClosestPointOnCollision, UPrimitiveComponent** OutPrimitiveComponent = nullptr) const;

	bool IsInLevel(const class ULevel *TestLevel) const;

	ULevel* GetLevel() const;

	FTransform GetLevelTransform() const;

	virtual void ClearCrossLevelReferences();


	virtual bool IsBasedOnActor(const AActor* Other) const;

	virtual bool IsAttachedTo( const AActor* Other ) const;

	FVector GetPlacementExtent() const;

	void ResetPropertiesForConstruction();

	virtual void RerunConstructionScripts();

	void DebugShowComponentHierarchy( const TCHAR* Info, bool bShowPosition  = true);

	void DebugShowOneComponentHierarchy( USceneComponent* SceneComp, int32& NestLevel, bool bShowPosition );

	bool ExecuteConstruction(const FTransform& Transform, const struct FRotationConversionCache* TransformRotationCache, const class FComponentInstanceDataCache* InstanceDataCache, bool bIsDefaultTransform = false);

	virtual void OnConstruction(const FTransform& Transform) {}

	void FinishAndRegisterComponent(UActorComponent* Component);

	UActorComponent* CreateComponentFromTemplate(UActorComponent* Template, const FName InName = NAME_None );
	UActorComponent* CreateComponentFromTemplateData(const struct FBlueprintCookedComponentInstancingData* TemplateData, const FName InName = NAME_None);

	void DestroyConstructedComponents();
protected:

	virtual void RegisterActorTickFunctions(bool bRegister);

	void ProcessUserConstructionScript();

	bool CheckActorComponents() const;

	void PostCreateBlueprintComponent(UActorComponent* NewActorComp);
public:

	void CheckComponentInstanceName(const FName InName);

	AActor* GetAttachParentActor() const;

	FName GetAttachParentSocketName() const;

	void ForEachAttachedActors(TFunctionRef<bool(class AActor*)> Functor) const;

	void GetAttachedActors(TArray<AActor*>& OutActors, bool bResetArray = true) const;

	void SetTickGroup(ETickingGroup NewTickGroup);

	virtual void Destroyed();

	void DispatchBlockingHit(UPrimitiveComponent* MyComp, UPrimitiveComponent* OtherComp, bool bSelfMoved, FHitResult const& Hit);

	virtual void FellOutOfWorld(const class UDamageType& dmgType);

	virtual void OutsideWorldBounds();

	virtual FBox GetComponentsBoundingBox(bool bNonColliding = false, bool bIncludeFromChildActors = false) const;

	virtual FBox CalculateComponentsBoundingBoxInLocalSpace(bool bNonColliding = false, bool bIncludeFromChildActors = false) const;

	virtual void GetComponentsBoundingCylinder(float& CollisionRadius, float& CollisionHalfHeight, bool bNonColliding = false, bool bIncludeFromChildActors = false) const;

	virtual void GetSimpleCollisionCylinder(float& CollisionRadius, float& CollisionHalfHeight) const;

	float GetSimpleCollisionRadius() const;

	float GetSimpleCollisionHalfHeight() const;

	FVector GetSimpleCollisionCylinderExtent() const;

	virtual bool IsRootComponentCollisionRegistered() const;

	virtual void TornOff();

	virtual ECollisionResponse GetComponentsCollisionResponseToChannel(ECollisionChannel Channel) const;

	void DisableComponentsSimulatePhysics();


	virtual bool CanBeBaseForCharacter(class APawn* Pawn) const;

	virtual float TakeDamage(float DamageAmount, struct FDamageEvent const& DamageEvent, class AController* EventInstigator, AActor* DamageCauser);
protected:
	virtual float InternalTakeRadialDamage(float Damage, struct FRadialDamageEvent const& RadialDamageEvent, class AController* EventInstigator, AActor* DamageCauser);
	virtual float InternalTakePointDamage(float Damage, struct FPointDamageEvent const& PointDamageEvent, class AController* EventInstigator, AActor* DamageCauser);
public:

	virtual void BecomeViewTarget( class APlayerController* PC );

	virtual void EndViewTarget( class APlayerController* PC );

	void K2_OnBecomeViewTarget( class APlayerController* PC );

	void K2_OnEndViewTarget( class APlayerController* PC );

	virtual void CalcCamera(float DeltaTime, struct FMinimalViewInfo& OutResult);

	virtual bool HasActiveCameraComponent() const;

	virtual bool HasActivePawnControlCameraComponent() const;

	virtual FString GetHumanReadableName() const;

	virtual void Reset();

	void K2_OnReset();

	bool WasRecentlyRendered(float Tolerance = 0.2) const;

	virtual float GetLastRenderTime() const;

	virtual void ForceNetRelevant();




	virtual void ForceNetUpdate();

	virtual void PrestreamTextures( float Seconds, bool bEnableStreaming, int32 CinematicTextureGroups = 0 );

	virtual void GetActorEyesViewPoint( FVector& OutLocation, FRotator& OutRotation ) const;

	virtual FVector GetTargetLocation(AActor* RequestedBy = nullptr) const;

	virtual void PostRenderFor(class APlayerController* PC, class UCanvas* Canvas, FVector CameraPosition, FVector CameraDir);

	bool IsInPersistentLevel(bool bIncludeLevelStreamingPersistent = false) const;

	virtual UWorld* GetWorld() const override;




	bool IsNetStartupActor() const;

	virtual UActorComponent* FindComponentByClass(const TSubclassOf<UActorComponent> ComponentClass) const;

	UActorComponent* GetComponentByClass(TSubclassOf<UActorComponent> ComponentClass) const;

	TArray<UActorComponent*> K2_GetComponentsByClass(TSubclassOf<UActorComponent> ComponentClass) const;

	TArray<UActorComponent*> GetComponentsByTag(TSubclassOf<UActorComponent> ComponentClass, FName Tag) const;

	TArray<UActorComponent*> GetComponentsByInterface(TSubclassOf<UInterface> Interface) const;

private:


public:






	const TSet<UActorComponent*>& GetComponents() const;

	void AddOwnedComponent(UActorComponent* Component);

	void RemoveOwnedComponent(UActorComponent* Component);

	bool OwnsComponent(UActorComponent* Component) const;

	void ResetOwnedComponents();

	void UpdateReplicatedComponent(UActorComponent* Component);

	void UpdateAllReplicatedComponents();

	 bool GetIsReplicated() const;

	const TArray<UActorComponent*>& GetReplicatedComponents() const;
protected:

	TArray<UActorComponent*> ReplicatedComponents;
private:

	TSet<UActorComponent*> OwnedComponents;

	TArray<TObjectPtr<UActorComponent>> InstanceComponents;
public:

	TArray<TObjectPtr<UActorComponent>> BlueprintCreatedComponents;

	void AddInstanceComponent(UActorComponent* Component);

	void RemoveInstanceComponent(UActorComponent* Component);

	void ClearInstanceComponents(bool bDestroyComponents);

	const TArray<UActorComponent*>& GetInstanceComponents() const;


	static void MakeNoiseImpl(AActor* NoiseMaker, float Loudness, APawn* NoiseInstigator, const FVector& NoiseLocation, float MaxRange, FName Tag);

	static void SetMakeNoiseDelegate(const FMakeNoiseDelegate& NewDelegate);

virtual bool IsComponentRelevantForNavigation(UActorComponent* Component) const ;
private:
	static FMakeNoiseDelegate MakeNoiseDelegate;
public:

	virtual void DisplayDebug(class UCanvas* Canvas, const class FDebugDisplayInfo& DebugDisplay, float& YL, float& YPos);

static FString GetDebugName(const AActor* Actor) ;

	static FOnProcessEvent ProcessEventDelegate;

	FRenderCommandFence DetachFence;
private:

	void InternalDispatchBlockingHit(UPrimitiveComponent* MyComp, UPrimitiveComponent* OtherComp, bool bSelfMoved, FHitResult const& Hit);

	ENetMode InternalGetNetMode() const;

	bool InternalPostEditUndo();
	friend struct FMarkActorIsBeingDestroyed;
	friend struct FActorParentComponentSetter;
	friend struct FSetActorWantsDestroyDuringBeginPlay;
public:

	static const FName GetHiddenPropertyName();

	bool IsHidden() const;

	void SetHidden(const bool bInHidden);

	static const FName GetReplicateMovementPropertyName();

	bool IsReplicatingMovement() const;

	void SetReplicatingMovement(bool bInReplicateMovement);

	static const FName GetCanBeDamagedPropertyName();

	bool CanBeDamaged() const;

	void SetCanBeDamaged(bool bInCanBeDamaged);

	static const FName GetRolePropertyName();

	void SetRole(ENetRole InRole);

	const FRepMovement& GetReplicatedMovement() const;

	FRepMovement& GetReplicatedMovement_Mutable();

	void SetReplicatedMovement(const FRepMovement& InReplicatedMovement);

	static const FName GetInstigatorPropertyName();

	void SetInstigator(APawn* InInstigator);
};

struct FMarkActorIsBeingDestroyed
{
private:
	FMarkActorIsBeingDestroyed(AActor* InActor);
	friend UWorld;
};

struct FSetActorWantsDestroyDuringBeginPlay
{
private:
	FSetActorWantsDestroyDuringBeginPlay(AActor* InActor);
	friend UWorld;
};

struct FActorLastRenderTime
{
private:
	static void Set(AActor* InActor, float LastRenderTime);
	static float* GetPtr(AActor* InActor);
};
